//! Applying an inlined rule to a goal: the soundness-critical computation
//! `next_goals = tactic(current_goal, args)`, modulo the equational theory.

use crate::core::name::{Interner, Sym};
use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Param};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::Expr;

/// Why `apply` rejected a step. Most variants are terse, already-readable
/// strings; the conclusion mismatch carries both propositions so a caller with
/// a name table can render them (`render`), while a caller without one still
/// gets a sensible one-line `Display`.
#[derive(Debug)]
pub enum ApplyError {
    /// The instantiated rule conclusion did not match the current goal.
    Mismatch { concluded: Expr, goal: Expr },
    /// Any other failure, already human-readable.
    Msg(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::Mismatch { .. } => {
                f.write_str("rule conclusion does not match the current goal")
            }
            ApplyError::Msg(s) => f.write_str(s),
        }
    }
}

impl From<String> for ApplyError {
    fn from(s: String) -> Self {
        ApplyError::Msg(s)
    }
}

impl From<&str> for ApplyError {
    fn from(s: &str) -> Self {
        ApplyError::Msg(s.to_string())
    }
}

impl ApplyError {
    /// Render the error with expression names resolved via `names`. Falls back
    /// to `Display` for variants that carry no expressions.
    pub fn render(&self, names: &Interner) -> String {
        match self {
            ApplyError::Mismatch { concluded, goal } => {
                use crate::core::display::show;
                format!(
                    "rule conclusion does not match the current goal\n  \
                     the rule concludes:  {}\n  \
                     but the goal is:     {}",
                    show(concluded, names),
                    show(goal, names),
                )
            }
            ApplyError::Msg(s) => s.clone(),
        }
    }
}

/// Apply `rule` with `args` against `current_goal` in context `parent_ctx`,
/// using the equational system `rs` for definitional equality. Returns the
/// generated premises (next goals) on success.
///
/// Performs the full soundness check: argument arity/kinds, proof-argument
/// statements, conclusion match, and the `forall_intro` side-condition.
pub fn apply(
    rule: &InlinedRule,
    args: &[Arg],
    current_goal: &Expr,
    parent_ctx: &[CtxEntry],
    rs: &RewriteSystem,
) -> Result<Vec<Sequent>, ApplyError> {
    if args.len() != rule.params.len() {
        return Err(format!(
            "tactic expects {} argument(s), got {}",
            rule.params.len(),
            args.len()
        )
        .into());
    }

    // Term-parameter substitution and argument-kind check.
    let mut subst: Vec<(Sym, Expr)> = Vec::new();
    for (p, a) in rule.params.iter().zip(args) {
        match (p, a) {
            (Param::Term { name, .. }, Arg::Term(v)) => subst.push((*name, v.clone())),
            (Param::Proof { .. }, Arg::Proof(_)) => {}
            (Param::Term { .. }, Arg::Proof(_)) => {
                return Err("expected a term argument but got a proof reference".into())
            }
            (Param::Proof { .. }, Arg::Term(_)) => {
                return Err("expected a proof reference but got a term argument".into())
            }
        }
    }

    // Each proof argument must prove the parameter's (substituted) prop.
    for (p, a) in rule.params.iter().zip(args) {
        if let (Param::Proof { prop, .. }, Arg::Proof(stmt)) = (p, a) {
            let expected = subst_all(prop, &subst);
            if !rs.defeq(&expected, stmt) {
                return Err("proof argument does not prove the required statement".into());
            }
        }
    }

    // The instantiated conclusion must match the current goal (backward); for
    // bidirectional congruence rules the single premise may match instead.
    let concl = subst_all(&rule.conclusion, &subst);
    if !rs.defeq(&concl, current_goal) {
        if rule.bidirectional && rule.premises.len() == 1 && rule.premises[0].ctx.is_empty() {
            let prem = subst_all(&rule.premises[0].goal, &subst);
            if rs.defeq(&prem, current_goal) {
                check_forall_intro(rule, args, parent_ctx)?;
                return Ok(vec![Sequent {
                    ctx: parent_ctx.to_vec(),
                    goal: rs.nf(&concl),
                }]);
            }
        }
        return Err(ApplyError::Mismatch {
            concluded: concl,
            goal: current_goal.clone(),
        });
    }

    check_forall_intro(rule, args, parent_ctx)?;

    // Build the next goals: parent context extended by each premise's context.
    // Goals are normalized so child steps see normal forms.
    let mut next = Vec::new();
    for prem in &rule.premises {
        let mut ctx = parent_ctx.to_vec();
        for e in &prem.ctx {
            ctx.push(nf_entry(&subst_entry(e, &subst), rs));
        }
        let goal = rs.nf(&subst_all(&prem.goal, &subst));
        next.push(Sequent { ctx, goal });
    }
    Ok(next)
}

/// Generalization side-condition (spec §4.15): the generalized variable must
/// not be free in any proof hypothesis of the current context.
fn check_forall_intro(rule: &InlinedRule, args: &[Arg], parent_ctx: &[CtxEntry]) -> Result<(), String> {
    if rule.is_forall_intro {
        if let Some(Arg::Term(Expr::Free(v))) = args.get(1) {
            for e in parent_ctx {
                if let CtxEntry::Proof { prop, .. } = e {
                    if prop.has_free(*v) {
                        return Err(
                            "forall_intro side-condition violated: variable is free in a hypothesis"
                                .into(),
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Substitute all term parameters into an expression simultaneously (so an
/// argument mentioning another parameter's name is not captured).
pub fn subst_all(e: &Expr, subst: &[(Sym, Expr)]) -> Expr {
    e.subst_many(subst)
}

fn nf_entry(e: &CtxEntry, rs: &RewriteSystem) -> CtxEntry {
    match e {
        CtxEntry::Term { name, ty } => CtxEntry::Term {
            name: *name,
            ty: rs.nf(ty),
        },
        CtxEntry::Proof { name, prop } => CtxEntry::Proof {
            name: *name,
            prop: rs.nf(prop),
        },
    }
}

fn subst_entry(e: &CtxEntry, subst: &[(Sym, Expr)]) -> CtxEntry {
    match e {
        CtxEntry::Term { name, ty } => CtxEntry::Term {
            name: *name,
            ty: subst_all(ty, subst),
        },
        CtxEntry::Proof { name, prop } => CtxEntry::Proof {
            name: *name,
            prop: subst_all(prop, subst),
        },
    }
}
