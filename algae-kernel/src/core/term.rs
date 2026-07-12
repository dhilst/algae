//! The unified, universe-aware kernel expression type.
//!
//! Locally nameless: binders (`Lam`/`Forall`/`Exists`) use de Bruijn indices,
//! while free variables (params, context entries, eigenvariables) and declared
//! constants are interned [`Sym`]s. Term, type, proposition and kind all share
//! one `Expr` (e.g. `lambda (X : Sort) st Result(X, E)` mixes the levels).

use crate::core::name::Sym;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expr {
    /// de Bruijn bound variable (0 = innermost binder).
    Bound(u32),
    /// A free (named) variable: param, context entry, or eigenvariable.
    Free(Sym),
    /// A declared constant: sort, sort-constructor, or operator.
    Const(Sym),
    /// n-ary application `f(a1, ..., an)`.
    App(Box<Expr>, Vec<Expr>),
    /// `lambda (_ : ty) st body`.
    Lam(Box<Expr>, Box<Expr>),
    /// Non-dependent function type `dom -> cod`.
    Arrow(Box<Expr>, Box<Expr>),
    /// Dependent function type `Pi (_ : dom) . cod` — the type of a polymorphic
    /// operator or a type-abstracting lambda. Distinct from the logical `Forall`
    /// (which is Prop-valued); `Pi` is a function type and may be applied.
    Pi(Box<Expr>, Box<Expr>),
    /// Product type `a * b * ...`.
    Product(Vec<Expr>),
    /// Sum type `a | b | ...`.
    Sum(Vec<Expr>),
    /// The universe of sorts.
    Sort,
    /// The type of propositions.
    Prop,
    /// `a = b`.
    Eq(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Implies(Box<Expr>, Box<Expr>),
    Iff(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    False,
    /// `forall (_ : ty) st body`.
    Forall(Box<Expr>, Box<Expr>),
    /// `exists (_ : ty) st body`.
    Exists(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn app(head: Expr, args: Vec<Expr>) -> Expr {
        if args.is_empty() {
            head
        } else {
            Expr::App(Box::new(head), args)
        }
    }

    /// Shift de Bruijn indices `>= cutoff` by `d`.
    pub fn shift(&self, d: i64, cutoff: u32) -> Expr {
        match self {
            Expr::Bound(i) => {
                if *i >= cutoff {
                    Expr::Bound((*i as i64 + d) as u32)
                } else {
                    Expr::Bound(*i)
                }
            }
            Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => self.clone(),
            Expr::App(f, args) => Expr::App(
                Box::new(f.shift(d, cutoff)),
                args.iter().map(|a| a.shift(d, cutoff)).collect(),
            ),
            Expr::Lam(ty, b) => Expr::Lam(
                Box::new(ty.shift(d, cutoff)),
                Box::new(b.shift(d, cutoff + 1)),
            ),
            Expr::Forall(ty, b) => Expr::Forall(
                Box::new(ty.shift(d, cutoff)),
                Box::new(b.shift(d, cutoff + 1)),
            ),
            Expr::Exists(ty, b) => Expr::Exists(
                Box::new(ty.shift(d, cutoff)),
                Box::new(b.shift(d, cutoff + 1)),
            ),
            Expr::Pi(ty, b) => Expr::Pi(
                Box::new(ty.shift(d, cutoff)),
                Box::new(b.shift(d, cutoff + 1)),
            ),
            Expr::Arrow(a, b) => {
                Expr::Arrow(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff)))
            }
            Expr::Product(xs) => Expr::Product(xs.iter().map(|x| x.shift(d, cutoff)).collect()),
            Expr::Sum(xs) => Expr::Sum(xs.iter().map(|x| x.shift(d, cutoff)).collect()),
            Expr::Eq(a, b) => Expr::Eq(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff))),
            Expr::And(a, b) => Expr::And(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff))),
            Expr::Or(a, b) => Expr::Or(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff))),
            Expr::Implies(a, b) => {
                Expr::Implies(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff)))
            }
            Expr::Iff(a, b) => Expr::Iff(Box::new(a.shift(d, cutoff)), Box::new(b.shift(d, cutoff))),
            Expr::Not(a) => Expr::Not(Box::new(a.shift(d, cutoff))),
        }
    }

    /// Whether de Bruijn index `idx` occurs free in `self`.
    pub fn has_bound(&self, idx: u32) -> bool {
        match self {
            Expr::Bound(i) => *i == idx,
            Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => false,
            Expr::App(f, args) => f.has_bound(idx) || args.iter().any(|a| a.has_bound(idx)),
            Expr::Lam(ty, b) | Expr::Forall(ty, b) | Expr::Exists(ty, b) | Expr::Pi(ty, b) => {
                ty.has_bound(idx) || b.has_bound(idx + 1)
            }
            Expr::Arrow(a, b)
            | Expr::Eq(a, b)
            | Expr::And(a, b)
            | Expr::Or(a, b)
            | Expr::Implies(a, b)
            | Expr::Iff(a, b) => a.has_bound(idx) || b.has_bound(idx),
            Expr::Product(xs) | Expr::Sum(xs) => xs.iter().any(|x| x.has_bound(idx)),
            Expr::Not(a) => a.has_bound(idx),
        }
    }

    /// Whether free variable `s` occurs in `self`.
    pub fn has_free(&self, s: Sym) -> bool {
        match self {
            Expr::Free(x) => *x == s,
            Expr::Bound(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => false,
            Expr::App(f, args) => f.has_free(s) || args.iter().any(|a| a.has_free(s)),
            Expr::Lam(ty, b) | Expr::Forall(ty, b) | Expr::Exists(ty, b) | Expr::Pi(ty, b) => {
                ty.has_free(s) || b.has_free(s)
            }
            Expr::Arrow(a, b)
            | Expr::Eq(a, b)
            | Expr::And(a, b)
            | Expr::Or(a, b)
            | Expr::Implies(a, b)
            | Expr::Iff(a, b) => a.has_free(s) || b.has_free(s),
            Expr::Product(xs) | Expr::Sum(xs) => xs.iter().any(|x| x.has_free(s)),
            Expr::Not(a) => a.has_free(s),
        }
    }

    /// Replace free variable `s` with `v` (capture-free: `v`'s only binders are
    /// its own de Bruijn binders, which cannot be captured).
    pub fn subst_free(&self, s: Sym, v: &Expr) -> Expr {
        match self {
            Expr::Free(x) if *x == s => v.clone(),
            Expr::Free(_) | Expr::Bound(_) | Expr::Const(_) | Expr::Sort | Expr::Prop
            | Expr::False => self.clone(),
            Expr::App(f, args) => Expr::App(
                Box::new(f.subst_free(s, v)),
                args.iter().map(|a| a.subst_free(s, v)).collect(),
            ),
            Expr::Lam(ty, b) => {
                Expr::Lam(Box::new(ty.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Forall(ty, b) => {
                Expr::Forall(Box::new(ty.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Exists(ty, b) => {
                Expr::Exists(Box::new(ty.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Pi(ty, b) => {
                Expr::Pi(Box::new(ty.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Arrow(a, b) => {
                Expr::Arrow(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Product(xs) => Expr::Product(xs.iter().map(|x| x.subst_free(s, v)).collect()),
            Expr::Sum(xs) => Expr::Sum(xs.iter().map(|x| x.subst_free(s, v)).collect()),
            Expr::Eq(a, b) => Expr::Eq(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v))),
            Expr::And(a, b) => Expr::And(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v))),
            Expr::Or(a, b) => Expr::Or(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v))),
            Expr::Implies(a, b) => {
                Expr::Implies(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v)))
            }
            Expr::Iff(a, b) => Expr::Iff(Box::new(a.subst_free(s, v)), Box::new(b.subst_free(s, v))),
            Expr::Not(a) => Expr::Not(Box::new(a.subst_free(s, v))),
        }
    }

    /// Rename free variable `from` to `to`.
    pub fn rename_free(&self, from: Sym, to: Sym) -> Expr {
        self.subst_free(from, &Expr::Free(to))
    }

    /// Simultaneously substitute free variables per `map`. Substituted values
    /// are inserted wholesale (not re-traversed), so an argument that happens to
    /// mention another parameter's name is not captured.
    pub fn subst_many(&self, map: &[(Sym, Expr)]) -> Expr {
        match self {
            Expr::Free(s) => map
                .iter()
                .find(|(n, _)| n == s)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| self.clone()),
            Expr::Bound(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => self.clone(),
            Expr::App(f, args) => Expr::App(
                Box::new(f.subst_many(map)),
                args.iter().map(|a| a.subst_many(map)).collect(),
            ),
            Expr::Lam(ty, b) => Expr::Lam(Box::new(ty.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::Pi(ty, b) => Expr::Pi(Box::new(ty.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::Forall(ty, b) => {
                Expr::Forall(Box::new(ty.subst_many(map)), Box::new(b.subst_many(map)))
            }
            Expr::Exists(ty, b) => {
                Expr::Exists(Box::new(ty.subst_many(map)), Box::new(b.subst_many(map)))
            }
            Expr::Arrow(a, b) => {
                Expr::Arrow(Box::new(a.subst_many(map)), Box::new(b.subst_many(map)))
            }
            Expr::Product(xs) => Expr::Product(xs.iter().map(|x| x.subst_many(map)).collect()),
            Expr::Sum(xs) => Expr::Sum(xs.iter().map(|x| x.subst_many(map)).collect()),
            Expr::Eq(a, b) => Expr::Eq(Box::new(a.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::And(a, b) => Expr::And(Box::new(a.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::Or(a, b) => Expr::Or(Box::new(a.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::Implies(a, b) => {
                Expr::Implies(Box::new(a.subst_many(map)), Box::new(b.subst_many(map)))
            }
            Expr::Iff(a, b) => Expr::Iff(Box::new(a.subst_many(map)), Box::new(b.subst_many(map))),
            Expr::Not(a) => Expr::Not(Box::new(a.subst_many(map))),
        }
    }
}

/// Open the outermost binder of `body` by substituting `v` for de Bruijn 0,
/// decrementing the remaining indices. This is the core of beta reduction.
pub fn open(body: &Expr, v: &Expr) -> Expr {
    subst_bound(body, 0, v)
}

fn subst_bound(e: &Expr, k: u32, v: &Expr) -> Expr {
    match e {
        Expr::Bound(i) => {
            if *i == k {
                // Shift v by k to account for the binders we have descended under.
                v.shift(k as i64, 0)
            } else if *i > k {
                Expr::Bound(*i - 1)
            } else {
                Expr::Bound(*i)
            }
        }
        Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => e.clone(),
        Expr::App(f, args) => Expr::App(
            Box::new(subst_bound(f, k, v)),
            args.iter().map(|a| subst_bound(a, k, v)).collect(),
        ),
        Expr::Lam(ty, b) => Expr::Lam(
            Box::new(subst_bound(ty, k, v)),
            Box::new(subst_bound(b, k + 1, v)),
        ),
        Expr::Forall(ty, b) => Expr::Forall(
            Box::new(subst_bound(ty, k, v)),
            Box::new(subst_bound(b, k + 1, v)),
        ),
        Expr::Exists(ty, b) => Expr::Exists(
            Box::new(subst_bound(ty, k, v)),
            Box::new(subst_bound(b, k + 1, v)),
        ),
        Expr::Pi(ty, b) => Expr::Pi(
            Box::new(subst_bound(ty, k, v)),
            Box::new(subst_bound(b, k + 1, v)),
        ),
        Expr::Arrow(a, b) => Expr::Arrow(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v))),
        Expr::Product(xs) => Expr::Product(xs.iter().map(|x| subst_bound(x, k, v)).collect()),
        Expr::Sum(xs) => Expr::Sum(xs.iter().map(|x| subst_bound(x, k, v)).collect()),
        Expr::Eq(a, b) => Expr::Eq(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v))),
        Expr::And(a, b) => Expr::And(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v))),
        Expr::Or(a, b) => Expr::Or(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v))),
        Expr::Implies(a, b) => {
            Expr::Implies(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v)))
        }
        Expr::Iff(a, b) => Expr::Iff(Box::new(subst_bound(a, k, v)), Box::new(subst_bound(b, k, v))),
        Expr::Not(a) => Expr::Not(Box::new(subst_bound(a, k, v))),
    }
}

/// Abstract free variable `s` into a fresh outermost de Bruijn binder (the
/// inverse of `open`). Used when building binders during elaboration.
pub fn close(e: &Expr, s: Sym) -> Expr {
    close_at(e, s, 0)
}

fn close_at(e: &Expr, s: Sym, k: u32) -> Expr {
    match e {
        Expr::Free(x) if *x == s => Expr::Bound(k),
        Expr::Free(_) | Expr::Bound(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => {
            e.clone()
        }
        Expr::App(f, args) => Expr::App(
            Box::new(close_at(f, s, k)),
            args.iter().map(|a| close_at(a, s, k)).collect(),
        ),
        Expr::Lam(ty, b) => Expr::Lam(Box::new(close_at(ty, s, k)), Box::new(close_at(b, s, k + 1))),
        Expr::Forall(ty, b) => {
            Expr::Forall(Box::new(close_at(ty, s, k)), Box::new(close_at(b, s, k + 1)))
        }
        Expr::Exists(ty, b) => {
            Expr::Exists(Box::new(close_at(ty, s, k)), Box::new(close_at(b, s, k + 1)))
        }
        Expr::Pi(ty, b) => {
            Expr::Pi(Box::new(close_at(ty, s, k)), Box::new(close_at(b, s, k + 1)))
        }
        Expr::Arrow(a, b) => Expr::Arrow(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k))),
        Expr::Product(xs) => Expr::Product(xs.iter().map(|x| close_at(x, s, k)).collect()),
        Expr::Sum(xs) => Expr::Sum(xs.iter().map(|x| close_at(x, s, k)).collect()),
        Expr::Eq(a, b) => Expr::Eq(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k))),
        Expr::And(a, b) => Expr::And(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k))),
        Expr::Or(a, b) => Expr::Or(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k))),
        Expr::Implies(a, b) => {
            Expr::Implies(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k)))
        }
        Expr::Iff(a, b) => Expr::Iff(Box::new(close_at(a, s, k)), Box::new(close_at(b, s, k))),
        Expr::Not(a) => Expr::Not(Box::new(close_at(a, s, k))),
    }
}
