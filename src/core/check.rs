//! The three-phase proof checker.
//!
//! 1. Read the proof steps (the [`Step`] tree).
//! 2. In parallel, verify each step locally: `next_goals == tactic(current_goal,
//!    args)` (recomputed from the inlined rule, never trusted).
//! 3. Verify linkage: each child's goal is the corresponding next goal, and
//!    leaves close their goal.

use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Step};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::tactic::apply;
use crate::core::term::Expr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use threadpool::ThreadPool;

/// Owned, self-contained data for one step's local (phase-2) check.
#[derive(Clone)]
struct StepData {
    context: Vec<CtxEntry>,
    current_goal: Expr,
    tactic: InlinedRule,
    args: Vec<Arg>,
    next_goals: Vec<Sequent>,
    label: String,
}

fn collect(step: &Step, label_of: &dyn Fn(&Step) -> String, out: &mut Vec<StepData>) {
    // Admitted (`by wip`) steps are assumed, not checked.
    if step.admitted {
        return;
    }
    out.push(StepData {
        context: step.context.clone(),
        current_goal: step.current_goal.clone(),
        tactic: step.tactic.clone(),
        args: step.args.clone(),
        next_goals: step.next_goals.clone(),
        label: label_of(step),
    });
    for c in &step.children {
        collect(c, label_of, out);
    }
}

/// Whether two sequents are equal up to definitional equality (same context
/// names, defeq types/props, defeq goal).
fn seq_defeq(a: &Sequent, b: &Sequent, rs: &RewriteSystem) -> bool {
    if a.ctx.len() != b.ctx.len() {
        return false;
    }
    for (x, y) in a.ctx.iter().zip(&b.ctx) {
        let ok = match (x, y) {
            (CtxEntry::Term { name: n1, ty: t1 }, CtxEntry::Term { name: n2, ty: t2 }) => {
                n1 == n2 && rs.defeq(t1, t2)
            }
            (CtxEntry::Proof { name: n1, prop: p1 }, CtxEntry::Proof { name: n2, prop: p2 }) => {
                n1 == n2 && rs.defeq(p1, p2)
            }
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    rs.defeq(&a.goal, &b.goal)
}

/// Phase 2: recompute the rule application and compare to the stored next goals.
fn check_step_local(d: &StepData, rs: &RewriteSystem) -> Option<String> {
    let recomputed = match apply(&d.tactic, &d.args, &d.current_goal, &d.context, rs) {
        Ok(g) => g,
        Err(e) => return Some(format!("{}: {e}", d.label)),
    };
    if recomputed.len() != d.next_goals.len() {
        return Some(format!(
            "{}: tactic produced {} subgoal(s) but {} were recorded",
            d.label,
            recomputed.len(),
            d.next_goals.len()
        ));
    }
    for (r, s) in recomputed.iter().zip(&d.next_goals) {
        if !seq_defeq(r, s, rs) {
            return Some(format!("{}: recomputed subgoal does not match recorded subgoal", d.label));
        }
    }
    None
}

/// Phase 3: structural linkage between a step and its children.
fn check_linkage(step: &Step, label: &str, rs: &RewriteSystem, errors: &mut Vec<String>) {
    // Admitted (`by wip`) steps close their goal by assumption.
    if step.admitted {
        return;
    }
    if step.children.len() != step.next_goals.len() {
        errors.push(format!(
            "{label}: {} subgoal(s) but {} case(s) provided",
            step.next_goals.len(),
            step.children.len()
        ));
        return;
    }
    for (i, (child, ng)) in step.children.iter().zip(&step.next_goals).enumerate() {
        let child_seq = Sequent {
            ctx: child.context.clone(),
            goal: child.current_goal.clone(),
        };
        if !seq_defeq(&child_seq, ng, rs) {
            errors.push(format!("{label}: case {i} does not match the generated subgoal"));
        }
        check_linkage(child, &format!("{label}.{i}"), rs, errors);
    }
}

/// Check a proof tree. `proof_label` names the lemma/law for diagnostics.
/// `jobs` is the worker count for the parallel phase.
pub fn check(root: &Step, proof_label: &str, jobs: usize, rs: &RewriteSystem) -> Vec<String> {
    // Phase 1: flatten the tree.
    let label_of = |s: &Step| -> String { format!("{proof_label}: step `{}`", s_label(s)) };
    let mut steps = Vec::new();
    collect(root, &label_of, &mut steps);

    // Phase 2: parallel local checks.
    let mut errors = Vec::new();
    let n = steps.len();
    if jobs <= 1 || n <= 1 {
        for d in &steps {
            if let Some(e) = check_step_local(d, rs) {
                errors.push(e);
            }
        }
    } else {
        let pool = ThreadPool::new(jobs.min(n));
        let shared = Arc::new(steps);
        let rs_shared = Arc::new(rs.clone());
        let (tx, rx) = channel();
        for i in 0..n {
            let shared = Arc::clone(&shared);
            let rs_shared = Arc::clone(&rs_shared);
            let tx = tx.clone();
            pool.execute(move || {
                let res = check_step_local(&shared[i], &rs_shared);
                tx.send(res).expect("send");
            });
        }
        drop(tx);
        for res in rx.iter() {
            if let Some(e) = res {
                errors.push(e);
            }
        }
        // Deterministic error order regardless of thread scheduling.
        errors.sort();
    }

    // Phase 3: linkage on the tree.
    check_linkage(root, proof_label, rs, &mut errors);
    errors
}

fn s_label(_s: &Step) -> String {
    String::new()
}
