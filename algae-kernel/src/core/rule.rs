//! Inference rules in inlined form, plus proof steps.
//!
//! Tactics (axioms, rules, lemmas/theorems, local hypotheses, theory laws) are
//! all represented as an [`InlinedRule`]: a list of parameters, premise
//! templates and a conclusion template. Parameters appear as `Free` variables
//! in the templates; applying the rule substitutes term arguments for them.

use crate::core::name::Sym;
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::Expr;
use crate::diagnostics::Span;

/// A rule parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Param {
    /// A term parameter `name : ty`, substituted by a term argument.
    Term { name: Sym, ty: Expr },
    /// A proof parameter `name := prop`; the argument is a proof reference whose
    /// statement must match `prop` (after earlier substitutions). Not
    /// substituted into premises/conclusion.
    Proof { name: Sym, prop: Expr },
}

impl Param {
    pub fn name(&self) -> Sym {
        match self {
            Param::Term { name, .. } | Param::Proof { name, .. } => *name,
        }
    }
}

/// An inference rule with everything needed to check an application inlined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlinedRule {
    pub params: Vec<Param>,
    /// Premise templates. Each premise's `ctx` is the *extension* added to the
    /// current goal's context (eigenvariables / hypotheses); `goal` is the
    /// premise proposition.
    pub premises: Vec<Sequent>,
    /// The conclusion proposition (the rule concludes `|- conclusion`).
    pub conclusion: Expr,
    /// True for the built-in `forall_intro` rule, which carries a
    /// side-condition (spec §4.15).
    pub is_forall_intro: bool,
    /// True for the congruence rewrite rules (`rewrite_r`/`rewrite_l`). Their
    /// single premise establishes a congruence under a verified equation, so
    /// the goal may match either the conclusion (standard) or the premise (the
    /// reverse rewrite); both are sound because `=` is symmetric.
    pub bidirectional: bool,
}

/// An argument supplied to a tactic, aligned positionally with the rule params.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    /// A term argument for a term parameter.
    Term(Expr),
    /// The (inlined) statement of a proof reference supplied for a proof
    /// parameter.
    Proof(Expr),
}

/// A fully-elaborated proof step. Self-contained: checking needs only the data
/// here, no environment lookups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The current context (named telescope) in which the goal is stated.
    pub context: Vec<CtxEntry>,
    /// The proposition being proved at this step.
    pub current_goal: Expr,
    /// The name of the tactic applied (for diagnostics).
    pub tactic_name: Sym,
    /// The inlined rule for the tactic.
    pub tactic: InlinedRule,
    /// Arguments aligned with `tactic.params`.
    pub args: Vec<Arg>,
    /// The premises this step generates, instantiated (context = current
    /// context extended by each premise's local context).
    pub next_goals: Vec<Sequent>,
    /// Sub-proofs: `children[i]` proves `next_goals[i]`.
    pub children: Vec<Step>,
    /// True for an admitted (`by wip`) leaf: the goal is assumed, not checked.
    pub admitted: bool,
    /// Source span of the `by` statement this step was elaborated from, for
    /// diagnostics.
    pub span: Span,
}
