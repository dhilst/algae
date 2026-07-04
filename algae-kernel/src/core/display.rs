//! Human-readable rendering of core [`Expr`] values for diagnostics.
//!
//! The core term language is locally nameless (de Bruijn indices for binders),
//! so rendering invents readable names for bound variables and resolves free
//! variables / constants through the [`Interner`]. Output uses the Unicode
//! operator glyphs. It is meant for messages (e.g. hole reports), not for
//! round-tripping, so it parenthesizes conservatively rather than minimally.

use crate::core::name::Interner;
use crate::core::term::Expr;

/// Render `e` to a readable string using `names` to resolve symbols.
pub fn show(e: &Expr, names: &Interner) -> String {
    let mut binders: Vec<String> = Vec::new();
    let mut out = String::new();
    go(e, names, &mut binders, &mut out);
    out
}

/// A pool of readable binder names, cycling with a numeric suffix when deep.
fn binder_name(depth: usize) -> String {
    const POOL: &[&str] = &["x", "y", "z", "u", "v", "w", "p", "q", "r", "s", "t"];
    let base = POOL[depth % POOL.len()];
    let round = depth / POOL.len();
    if round == 0 {
        base.to_string()
    } else {
        format!("{base}{round}")
    }
}

/// Whether an expression renders as a single atom (no surrounding parens needed
/// when it appears as an operand).
fn is_atom(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Bound(_)
            | Expr::Free(_)
            | Expr::Const(_)
            | Expr::App(..)
            | Expr::Sort
            | Expr::Prop
            | Expr::False
            | Expr::Not(_)
    )
}

fn operand(e: &Expr, names: &Interner, binders: &mut Vec<String>, out: &mut String) {
    if is_atom(e) {
        go(e, names, binders, out);
    } else {
        out.push('(');
        go(e, names, binders, out);
        out.push(')');
    }
}

fn binary(
    a: &Expr,
    op: &str,
    b: &Expr,
    names: &Interner,
    binders: &mut Vec<String>,
    out: &mut String,
) {
    operand(a, names, binders, out);
    out.push_str(op);
    operand(b, names, binders, out);
}

fn quantifier(
    kw: &str,
    ty: &Expr,
    body: &Expr,
    names: &Interner,
    binders: &mut Vec<String>,
    out: &mut String,
) {
    let name = binder_name(binders.len());
    out.push_str(kw);
    out.push_str(" (");
    out.push_str(&name);
    out.push_str(" : ");
    go(ty, names, binders, out);
    out.push_str(") st ");
    binders.push(name);
    go(body, names, binders, out);
    binders.pop();
}

fn go(e: &Expr, names: &Interner, binders: &mut Vec<String>, out: &mut String) {
    match e {
        Expr::Bound(i) => {
            let idx = binders.len().checked_sub(1 + *i as usize);
            match idx.and_then(|k| binders.get(k)) {
                Some(n) => out.push_str(n),
                None => out.push_str(&format!("?{i}")),
            }
        }
        Expr::Free(s) | Expr::Const(s) => out.push_str(names.resolve(*s)),
        Expr::App(f, args) => {
            // A symbolic binary operator (e.g. `+`, `×`) renders infix.
            if let Expr::Const(s) = f.as_ref() {
                let op = names.resolve(*s);
                if args.len() == 2 && op.chars().next().is_some_and(|c| !c.is_alphanumeric() && c != '_') {
                    operand(&args[0], names, binders, out);
                    out.push(' ');
                    out.push_str(op);
                    out.push(' ');
                    operand(&args[1], names, binders, out);
                    return;
                }
            }
            operand(f, names, binders, out);
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                go(a, names, binders, out);
            }
            out.push(')');
        }
        Expr::Lam(ty, b) => quantifier("λ", ty, b, names, binders, out),
        Expr::Forall(ty, b) => quantifier("∀", ty, b, names, binders, out),
        Expr::Exists(ty, b) => quantifier("∃", ty, b, names, binders, out),
        Expr::Arrow(a, b) => binary(a, " → ", b, names, binders, out),
        Expr::Product(xs) => join(xs, " × ", names, binders, out),
        Expr::Sum(xs) => join(xs, " | ", names, binders, out),
        Expr::Eq(a, b) => binary(a, " = ", b, names, binders, out),
        Expr::And(a, b) => binary(a, " ∧ ", b, names, binders, out),
        Expr::Or(a, b) => binary(a, " ∨ ", b, names, binders, out),
        Expr::Implies(a, b) => binary(a, " ⇒ ", b, names, binders, out),
        Expr::Iff(a, b) => binary(a, " ⇔ ", b, names, binders, out),
        Expr::Not(a) => {
            out.push('¬');
            operand(a, names, binders, out);
        }
        Expr::Sort => out.push_str("Sort"),
        Expr::Prop => out.push_str("Prop"),
        Expr::False => out.push_str("False"),
    }
}

fn join(xs: &[Expr], sep: &str, names: &Interner, binders: &mut Vec<String>, out: &mut String) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        operand(x, names, binders, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_infix_eq_and_quantifier() {
        let mut names = Interner::new();
        let nat = names.intern("Nat");
        let plus = names.intern("+");
        let zero = names.intern("0");
        let n = names.intern("n");
        // 0 + n = n
        let eq = Expr::Eq(
            Box::new(Expr::App(
                Box::new(Expr::Const(plus)),
                vec![Expr::Const(zero), Expr::Free(n)],
            )),
            Box::new(Expr::Free(n)),
        );
        assert_eq!(show(&eq, &names), "0 + n = n");
        // forall (x : Nat) st Bound(0) = Bound(0)
        let body = Expr::Eq(Box::new(Expr::Bound(0)), Box::new(Expr::Bound(0)));
        let all = Expr::Forall(Box::new(Expr::Const(nat)), Box::new(body));
        assert_eq!(show(&all, &names), "∀ (x : Nat) st x = x");
    }
}
