//! Equational normalization modulo the theory's equational axioms.
//!
//! Operators in Algae are defined by equational axioms (e.g.
//! `bind(none, f) = none`, `singleton(x) = cons(x, nil)`). The checker treats
//! these axioms as definitional rewrite rules `L -> R` (sound, since axioms are
//! assumed true) and normalizes terms by beta-reduction plus these rules. Two
//! terms are definitionally equal when their normal forms coincide.

use crate::core::name::Sym;
use crate::core::normalize::nf;
use crate::core::term::Expr;

/// A set of oriented equational rewrite rules.
#[derive(Clone, Debug, Default)]
pub struct RewriteSystem {
    /// Each rule is `(lhs, rhs, metavars)`; `metavars` are the rule's parameters
    /// (treated as wildcards during matching).
    pub rules: Vec<(Expr, Expr, Vec<Sym>)>,
}

impl RewriteSystem {
    pub fn new() -> RewriteSystem {
        RewriteSystem::default()
    }

    pub fn push(&mut self, lhs: Expr, rhs: Expr, metas: Vec<Sym>) {
        self.rules.push((lhs, rhs, metas));
    }

    /// Beta + equational normal form.
    pub fn nf(&self, e: &Expr) -> Expr {
        let mut cur = nf(e);
        for _ in 0..10_000 {
            let mut changed = false;
            for (l, r, metas) in &self.rules {
                let (next, c) = rewrite_pass(l, r, metas, &cur);
                if c {
                    cur = nf(&next);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        cur
    }

    /// Definitional equality modulo the equational rules.
    pub fn defeq(&self, a: &Expr, b: &Expr) -> bool {
        self.nf(a) == self.nf(b)
    }
}

/// First-order matching of `pat` (with `metas` as wildcards) against `term`.
pub fn match_pattern(pat: &Expr, term: &Expr, metas: &[Sym], subst: &mut Vec<(Sym, Expr)>) -> bool {
    if let Expr::Free(s) = pat {
        if metas.contains(s) {
            if let Some((_, prev)) = subst.iter().find(|(n, _)| n == s) {
                return prev == term;
            }
            subst.push((*s, term.clone()));
            return true;
        }
    }
    match (pat, term) {
        (Expr::Free(a), Expr::Free(b)) => a == b,
        (Expr::Bound(a), Expr::Bound(b)) => a == b,
        (Expr::Const(a), Expr::Const(b)) => a == b,
        (Expr::Sort, Expr::Sort) | (Expr::Prop, Expr::Prop) | (Expr::False, Expr::False) => true,
        (Expr::App(f1, a1), Expr::App(f2, a2)) => {
            a1.len() == a2.len()
                && match_pattern(f1, f2, metas, subst)
                && a1.iter().zip(a2).all(|(x, y)| match_pattern(x, y, metas, subst))
        }
        (Expr::Lam(t1, b1), Expr::Lam(t2, b2))
        | (Expr::Forall(t1, b1), Expr::Forall(t2, b2))
        | (Expr::Exists(t1, b1), Expr::Exists(t2, b2)) => {
            match_pattern(t1, t2, metas, subst) && match_pattern(b1, b2, metas, subst)
        }
        (Expr::Arrow(a1, b1), Expr::Arrow(a2, b2))
        | (Expr::Eq(a1, b1), Expr::Eq(a2, b2))
        | (Expr::And(a1, b1), Expr::And(a2, b2))
        | (Expr::Or(a1, b1), Expr::Or(a2, b2))
        | (Expr::Implies(a1, b1), Expr::Implies(a2, b2))
        | (Expr::Iff(a1, b1), Expr::Iff(a2, b2)) => {
            match_pattern(a1, a2, metas, subst) && match_pattern(b1, b2, metas, subst)
        }
        (Expr::Product(a), Expr::Product(b)) | (Expr::Sum(a), Expr::Sum(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| match_pattern(x, y, metas, subst))
        }
        (Expr::Not(a), Expr::Not(b)) => match_pattern(a, b, metas, subst),
        _ => false,
    }
}

fn subst_metas(e: &Expr, subst: &[(Sym, Expr)]) -> Expr {
    e.subst_many(subst)
}

/// One bottom-up rewrite pass with `l -> r`.
fn rewrite_pass(l: &Expr, r: &Expr, metas: &[Sym], e: &Expr) -> (Expr, bool) {
    let (e, changed) = rewrite_children(l, r, metas, e);
    let mut subst = Vec::new();
    if match_pattern(l, &e, metas, &mut subst) {
        let result = subst_metas(r, &subst);
        if result != e {
            return (result, true);
        }
    }
    (e, changed)
}

fn rewrite_children(l: &Expr, r: &Expr, metas: &[Sym], e: &Expr) -> (Expr, bool) {
    let mut changed = false;
    let mut rw = |x: &Expr| {
        let (nx, c) = rewrite_pass(l, r, metas, x);
        changed |= c;
        nx
    };
    let out = match e {
        Expr::App(f, args) => Expr::App(Box::new(rw(f)), args.iter().map(&mut rw).collect()),
        Expr::Lam(t, b) => Expr::Lam(Box::new(rw(t)), Box::new(rw(b))),
        Expr::Forall(t, b) => Expr::Forall(Box::new(rw(t)), Box::new(rw(b))),
        Expr::Exists(t, b) => Expr::Exists(Box::new(rw(t)), Box::new(rw(b))),
        Expr::Arrow(a, b) => Expr::Arrow(Box::new(rw(a)), Box::new(rw(b))),
        Expr::Eq(a, b) => Expr::Eq(Box::new(rw(a)), Box::new(rw(b))),
        Expr::And(a, b) => Expr::And(Box::new(rw(a)), Box::new(rw(b))),
        Expr::Or(a, b) => Expr::Or(Box::new(rw(a)), Box::new(rw(b))),
        Expr::Implies(a, b) => Expr::Implies(Box::new(rw(a)), Box::new(rw(b))),
        Expr::Iff(a, b) => Expr::Iff(Box::new(rw(a)), Box::new(rw(b))),
        Expr::Not(a) => Expr::Not(Box::new(rw(a))),
        Expr::Product(xs) => Expr::Product(xs.iter().map(&mut rw).collect()),
        Expr::Sum(xs) => Expr::Sum(xs.iter().map(&mut rw).collect()),
        Expr::Bound(_) | Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => {
            e.clone()
        }
    };
    (out, changed)
}
