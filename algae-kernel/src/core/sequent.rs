//! Sequents and contexts (telescopes) over the kernel [`Expr`].
//!
//! A context entry binds a named free variable (a term variable or a proof
//! hypothesis). Eigenvariables and formal parameters are represented this way.

use crate::core::name::Sym;
use crate::core::term::Expr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtxEntry {
    /// A term variable `name : ty`.
    Term { name: Sym, ty: Expr },
    /// A proof hypothesis `name := prop`.
    Proof { name: Sym, prop: Expr },
}

impl CtxEntry {
    pub fn name(&self) -> Sym {
        match self {
            CtxEntry::Term { name, .. } | CtxEntry::Proof { name, .. } => *name,
        }
    }
}

/// A sequent `ctx |- goal`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequent {
    pub ctx: Vec<CtxEntry>,
    pub goal: Expr,
}

impl Sequent {
    pub fn new(ctx: Vec<CtxEntry>, goal: Expr) -> Sequent {
        Sequent { ctx, goal }
    }

    /// The names bound by the context.
    pub fn ctx_names(&self) -> Vec<Sym> {
        self.ctx.iter().map(|e| e.name()).collect()
    }
}
