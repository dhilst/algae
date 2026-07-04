//! The three-phase proof checker.
//!
//! 1. Read the proof steps (the [`Step`] tree).
//! 2. Verify each step locally: `next_goals == tactic(current_goal, args)`
//!    (recomputed from the inlined rule, never trusted).
//! 3. Verify linkage: each child's goal is the corresponding next goal, and
//!    leaves close their goal.

use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Step};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::tactic::apply;
use crate::core::term::Expr;
use crate::diagnostics::{Diagnostic, Span};

/// Owned, self-contained data for one step's local (phase-2) check.
struct StepData {
    context: Vec<CtxEntry>,
    current_goal: Expr,
    tactic: InlinedRule,
    args: Vec<Arg>,
    next_goals: Vec<Sequent>,
    label: String,
    span: Span,
}

fn collect(step: &Step, label: &str, out: &mut Vec<StepData>) {
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
        label: label.to_string(),
        span: step.span,
    });
    for c in &step.children {
        collect(c, label, out);
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

/// Build a proof-check diagnostic: `<label>: <reason>` anchored at `span`.
fn diag(label: &str, span: Span, reason: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::error(format!("{label}: {reason}")).with_span(span)
}

/// Phase 2: recompute the rule application and compare to the stored next goals.
fn check_step_local(d: &StepData, rs: &RewriteSystem) -> Option<Diagnostic> {
    let recomputed = match apply(&d.tactic, &d.args, &d.current_goal, &d.context, rs) {
        Ok(g) => g,
        Err(e) => return Some(diag(&d.label, d.span, e)),
    };
    if recomputed.len() != d.next_goals.len() {
        return Some(diag(
            &d.label,
            d.span,
            format!(
                "tactic produced {} subgoal(s) but {} were recorded",
                recomputed.len(),
                d.next_goals.len()
            ),
        ));
    }
    for (r, s) in recomputed.iter().zip(&d.next_goals) {
        if !seq_defeq(r, s, rs) {
            return Some(diag(&d.label, d.span, "recomputed subgoal does not match recorded subgoal"));
        }
    }
    None
}

/// Phase 3: structural linkage between a step and its children.
fn check_linkage(step: &Step, label: &str, rs: &RewriteSystem, errors: &mut Vec<Diagnostic>) {
    // Admitted (`by wip`) steps close their goal by assumption.
    if step.admitted {
        return;
    }
    if step.children.len() != step.next_goals.len() {
        errors.push(diag(
            label,
            step.span,
            format!(
                "{} subgoal(s) but {} case(s) provided",
                step.next_goals.len(),
                step.children.len()
            ),
        ));
        return;
    }
    for (i, (child, ng)) in step.children.iter().zip(&step.next_goals).enumerate() {
        let child_seq = Sequent {
            ctx: child.context.clone(),
            goal: child.current_goal.clone(),
        };
        if !seq_defeq(&child_seq, ng, rs) {
            errors.push(diag(
                label,
                child.span,
                format!("case {i} does not match the generated subgoal"),
            ));
        }
        check_linkage(child, &format!("{label}.{i}"), rs, errors);
    }
}

/// Check a proof tree. `proof_label` names the lemma/law for diagnostics.
pub fn check(root: &Step, proof_label: &str, rs: &RewriteSystem) -> Vec<Diagnostic> {
    // Phase 1: flatten the tree.
    let mut steps = Vec::new();
    collect(root, proof_label, &mut steps);

    // Phase 2: local checks.
    let mut errors = Vec::new();
    for d in &steps {
        if let Some(e) = check_step_local(d, rs) {
            errors.push(e);
        }
    }

    // Phase 3: linkage on the tree.
    check_linkage(root, proof_label, rs, &mut errors);
    errors
}
