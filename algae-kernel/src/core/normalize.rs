//! Beta normalization and definitional equality.
//!
//! The term language is simply typed (operators are inert constants — there is
//! **no** arithmetic evaluation), so beta-reduction is strongly normalizing and
//! a unique normal form exists. De Bruijn indices make alpha-equivalence the
//! same as structural equality, so definitional equality is "normalize both,
//! compare structurally".

use crate::core::term::{open, Expr};

/// Full beta normal form.
pub fn nf(e: &Expr) -> Expr {
    match e {
        Expr::App(f, args) => {
            let f = nf(f);
            let args: Vec<Expr> = args.iter().map(nf).collect();
            apply(f, args)
        }
        Expr::Lam(ty, b) => Expr::Lam(Box::new(nf(ty)), Box::new(nf(b))),
        Expr::Forall(ty, b) => Expr::Forall(Box::new(nf(ty)), Box::new(nf(b))),
        Expr::Exists(ty, b) => Expr::Exists(Box::new(nf(ty)), Box::new(nf(b))),
        Expr::Arrow(a, b) => Expr::Arrow(Box::new(nf(a)), Box::new(nf(b))),
        Expr::Product(xs) => Expr::Product(xs.iter().map(nf).collect()),
        Expr::Sum(xs) => Expr::Sum(xs.iter().map(nf).collect()),
        Expr::Eq(a, b) => Expr::Eq(Box::new(nf(a)), Box::new(nf(b))),
        Expr::And(a, b) => Expr::And(Box::new(nf(a)), Box::new(nf(b))),
        Expr::Or(a, b) => Expr::Or(Box::new(nf(a)), Box::new(nf(b))),
        Expr::Implies(a, b) => Expr::Implies(Box::new(nf(a)), Box::new(nf(b))),
        Expr::Iff(a, b) => Expr::Iff(Box::new(nf(a)), Box::new(nf(b))),
        Expr::Not(a) => Expr::Not(Box::new(nf(a))),
        Expr::Bound(_) | Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => {
            e.clone()
        }
    }
}

/// Apply a normalized head to normalized arguments, performing beta reduction.
fn apply(f: Expr, args: Vec<Expr>) -> Expr {
    if args.is_empty() {
        return f;
    }
    match f {
        Expr::Lam(_ty, body) => {
            let reduced = nf(&open(&body, &args[0]));
            apply(reduced, args[1..].to_vec())
        }
        Expr::App(g, mut gargs) => {
            // f is already a neutral application; extend with the new arguments.
            gargs.extend(args);
            Expr::App(g, gargs)
        }
        other => Expr::App(Box::new(other), args),
    }
}

/// Definitional equality: alpha/beta equivalence.
pub fn defeq(a: &Expr, b: &Expr) -> bool {
    nf(a) == nf(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::name::Sym;
    use crate::core::term::close;

    fn c(n: u32) -> Expr {
        Expr::Const(Sym(n))
    }
    fn f(n: u32) -> Expr {
        Expr::Free(Sym(n))
    }

    #[test]
    fn beta_reduces() {
        // (lambda (x : T) st x)(a)  ==>  a
        let id = Expr::Lam(Box::new(c(0)), Box::new(close(&f(100), Sym(100))));
        let app = Expr::App(Box::new(id), vec![c(5)]);
        assert_eq!(nf(&app), c(5));
    }

    #[test]
    fn beta_under_predicate() {
        // P := lambda (o : T) st o = a ; P(b) ==> b = a
        let body = Expr::Eq(Box::new(f(100)), Box::new(c(1)));
        let p = Expr::Lam(Box::new(c(0)), Box::new(close(&body, Sym(100))));
        let app = Expr::App(Box::new(p), vec![c(2)]);
        let expected = Expr::Eq(Box::new(c(2)), Box::new(c(1)));
        assert_eq!(nf(&app), expected);
        assert!(defeq(&app, &expected));
    }

    #[test]
    fn alpha_equivalence_is_structural() {
        // forall over T of (x = x), regardless of the original bound name.
        let p1 = Expr::Forall(Box::new(c(0)), Box::new(close(&Expr::Eq(Box::new(f(7)), Box::new(f(7))), Sym(7))));
        let p2 = Expr::Forall(Box::new(c(0)), Box::new(close(&Expr::Eq(Box::new(f(8)), Box::new(f(8))), Sym(8))));
        assert_eq!(nf(&p1), nf(&p2));
    }

    #[test]
    fn operators_are_inert() {
        // plus(0, 0) does not reduce to 0.
        let plus_00 = Expr::App(Box::new(c(10)), vec![c(11), c(11)]);
        assert_ne!(nf(&plus_00), c(11));
    }
}
