//! Parser built with `winnow` combinators over the token stream (`&[Token]`).
//!
//! The grammar follows spec §3. Dispatch on leading keywords is done by manual
//! lookahead; lists, optionals and alternation use winnow combinators.

use crate::diagnostics::{Diagnostic, Span};
use crate::parse::ast::*;
use crate::parse::lexer::{Token, TokenKind as T};
use winnow::combinator::separated;
use winnow::error::{ContextError, ErrMode, ParseError};
use winnow::{ModalResult, Parser};

type In<'a> = &'a [Token];

// ---- low-level helpers ----------------------------------------------------

fn backtrack<O>() -> ModalResult<O> {
    Err(ErrMode::Backtrack(ContextError::new()))
}

fn cur_span(input: In) -> Span {
    input.first().map(|t| t.span).unwrap_or_default()
}

fn at(input: In, k: &T) -> bool {
    matches!(input.first(), Some(t) if &t.kind == k)
}

fn peek_kind<'a>(input: In<'a>) -> Option<&'a T> {
    input.first().map(|t| &t.kind)
}

/// Consume the next token if its kind equals `k`, returning its span.
fn expect<'a>(input: &mut In<'a>, k: T) -> ModalResult<Span> {
    match input.first() {
        Some(t) if t.kind == k => {
            let s = t.span;
            *input = &input[1..];
            Ok(s)
        }
        _ => backtrack(),
    }
}

fn semi(i: &mut In) -> ModalResult<Span> {
    expect(i, T::Semi)
}
fn comma(i: &mut In) -> ModalResult<Span> {
    expect(i, T::Comma)
}

fn ident(input: &mut In) -> ModalResult<Name> {
    match input.first() {
        Some(Token {
            kind: T::Ident(s),
            span,
        }) => {
            let name = Name {
                text: s.clone(),
                span: *span,
            };
            *input = &input[1..];
            Ok(name)
        }
        _ => backtrack(),
    }
}

/// A name used as a binder/operator symbol: identifier, symbolic operator, or
/// numeric symbol — all normalized to a [`Name`] carrying its spelling.
fn bind_name(input: &mut In) -> ModalResult<Name> {
    if let Some(t) = input.first() {
        let text = match &t.kind {
            T::Ident(s) => s.clone(),
            T::Number(s) => s.clone(),
            T::Plus => "+".into(),
            T::Minus => "-".into(),
            T::Star => "*".into(),
            T::Slash => "/".into(),
            T::EqEq => "==".into(),
            T::Lt => "<".into(),
            T::Gt => ">".into(),
            T::Le => "<=".into(),
            T::Ge => ">=".into(),
            _ => return backtrack(),
        };
        let name = Name { text, span: t.span };
        *input = &input[1..];
        Ok(name)
    } else {
        backtrack()
    }
}

fn qname(input: &mut In) -> ModalResult<QName> {
    let first = ident(input)?;
    if at(*input, &T::Dot) {
        expect(input, T::Dot)?;
        let second = ident(input)?;
        let span = first.span.merge(second.span);
        Ok(QName {
            module: Some(first),
            name: second,
            span,
        })
    } else {
        let span = first.span;
        Ok(QName {
            module: None,
            name: first,
            span,
        })
    }
}

// ---- kinds ----------------------------------------------------------------

fn kind_atom(input: &mut In) -> ModalResult<Kind> {
    if at(*input, &T::KwSortU) {
        let s = expect(input, T::KwSortU)?;
        Ok(Kind {
            node: KindNode::Sort,
            span: s,
        })
    } else if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let k = kind_expr(input)?;
        expect(input, T::RParen)?;
        Ok(k)
    } else {
        backtrack()
    }
}

fn kind_product(input: &mut In) -> ModalResult<Kind> {
    let first = kind_atom(input)?;
    let mut parts = vec![first];
    while at(*input, &T::Star) {
        expect(input, T::Star)?;
        parts.push(kind_atom(input)?);
    }
    if parts.len() == 1 {
        Ok(parts.pop().unwrap())
    } else {
        let span = parts[0].span.merge(parts[parts.len() - 1].span);
        Ok(Kind {
            node: KindNode::Product(parts),
            span,
        })
    }
}

fn kind_expr(input: &mut In) -> ModalResult<Kind> {
    let left = kind_product(input)?;
    if at(*input, &T::Arrow) {
        expect(input, T::Arrow)?;
        let right = kind_expr(input)?;
        let span = left.span.merge(right.span);
        Ok(Kind {
            node: KindNode::Arrow(Box::new(left), Box::new(right)),
            span,
        })
    } else {
        Ok(left)
    }
}

// ---- types ----------------------------------------------------------------

fn type_atom(input: &mut In) -> ModalResult<Type> {
    if at(*input, &T::KwProp) {
        let s = expect(input, T::KwProp)?;
        return Ok(Type {
            node: TypeNode::Prop,
            span: s,
        });
    }
    if at(*input, &T::KwSortU) {
        let s = expect(input, T::KwSortU)?;
        return Ok(Type {
            node: TypeNode::Sort,
            span: s,
        });
    }
    if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let t = type_expr(input)?;
        expect(input, T::RParen)?;
        return Ok(t);
    }
    let q = qname(input)?;
    if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let args: Vec<Type> = separated(0.., type_expr, comma).parse_next(input)?;
        let close = expect(input, T::RParen)?;
        let span = q.span.merge(close);
        Ok(Type {
            node: TypeNode::App(q, args),
            span,
        })
    } else {
        let span = q.span;
        Ok(Type {
            node: TypeNode::Name(q),
            span,
        })
    }
}

fn type_product(input: &mut In) -> ModalResult<Type> {
    let first = type_atom(input)?;
    let mut parts = vec![first];
    while at(*input, &T::Star) {
        expect(input, T::Star)?;
        parts.push(type_atom(input)?);
    }
    if parts.len() == 1 {
        Ok(parts.pop().unwrap())
    } else {
        let span = parts[0].span.merge(parts[parts.len() - 1].span);
        Ok(Type {
            node: TypeNode::Product(parts),
            span,
        })
    }
}

fn type_sum(input: &mut In) -> ModalResult<Type> {
    let first = type_product(input)?;
    let mut parts = vec![first];
    while at(*input, &T::Bar) {
        expect(input, T::Bar)?;
        parts.push(type_product(input)?);
    }
    if parts.len() == 1 {
        Ok(parts.pop().unwrap())
    } else {
        let span = parts[0].span.merge(parts[parts.len() - 1].span);
        Ok(Type {
            node: TypeNode::Sum(parts),
            span,
        })
    }
}

fn type_expr(input: &mut In) -> ModalResult<Type> {
    let left = type_sum(input)?;
    if at(*input, &T::Arrow) {
        expect(input, T::Arrow)?;
        let right = type_expr(input)?;
        let span = left.span.merge(right.span);
        Ok(Type {
            node: TypeNode::Arrow(Box::new(left), Box::new(right)),
            span,
        })
    } else {
        Ok(left)
    }
}

// ---- binders & terms ------------------------------------------------------

fn binder(input: &mut In) -> ModalResult<Binder> {
    let open = expect(input, T::LParen)?;
    let names: Vec<Name> = repeat_min1(input, ident)?;
    expect(input, T::Colon)?;
    let ty = type_expr(input)?;
    let close = expect(input, T::RParen)?;
    Ok(Binder {
        names,
        ty,
        span: open.merge(close),
    })
}

/// `repeat(1.., p)` helper that stops on first failure.
fn repeat_min1<'a, O>(
    input: &mut In<'a>,
    mut p: impl FnMut(&mut In<'a>) -> ModalResult<O>,
) -> ModalResult<Vec<O>> {
    let mut out = vec![p(input)?];
    loop {
        let mut probe = *input;
        match p(&mut probe) {
            Ok(v) => {
                *input = probe;
                out.push(v);
            }
            Err(_) => break,
        }
    }
    Ok(out)
}

fn symop_kind(k: &T) -> Option<SymOp> {
    Some(match k {
        T::Plus => SymOp::Plus,
        T::Minus => SymOp::Minus,
        T::Star => SymOp::Star,
        T::Slash => SymOp::Slash,
        T::EqEq => SymOp::EqEq,
        T::Lt => SymOp::Lt,
        T::Gt => SymOp::Gt,
        T::Le => SymOp::Le,
        T::Ge => SymOp::Ge,
        _ => return None,
    })
}

// ---- expressions (terms and propositions unified) -------------------------
//
// Precedence, weakest to strongest (spec §3.21 ladder extended with terms):
//   <=>  =>  \/  /\  =  (~ | forall | exists | lambda)  (+ - * /)  application  atom

fn expr(input: &mut In) -> ModalResult<Expr> {
    e_iff(input)
}

fn e_iff(input: &mut In) -> ModalResult<Expr> {
    let left = e_implies(input)?;
    if at(*input, &T::Iff) {
        expect(input, T::Iff)?;
        let right = e_iff(input)?;
        let span = left.span.merge(right.span);
        Ok(Expr {
            node: ExprNode::Iff(Box::new(left), Box::new(right)),
            span,
        })
    } else {
        Ok(left)
    }
}

fn e_implies(input: &mut In) -> ModalResult<Expr> {
    let left = e_or(input)?;
    if at(*input, &T::Implies) {
        expect(input, T::Implies)?;
        let right = e_implies(input)?;
        let span = left.span.merge(right.span);
        Ok(Expr {
            node: ExprNode::Implies(Box::new(left), Box::new(right)),
            span,
        })
    } else {
        Ok(left)
    }
}

fn e_or(input: &mut In) -> ModalResult<Expr> {
    let mut left = e_and(input)?;
    while at(*input, &T::Or) {
        expect(input, T::Or)?;
        let right = e_and(input)?;
        let span = left.span.merge(right.span);
        left = Expr {
            node: ExprNode::Or(Box::new(left), Box::new(right)),
            span,
        };
    }
    Ok(left)
}

fn e_and(input: &mut In) -> ModalResult<Expr> {
    let mut left = e_eq(input)?;
    while at(*input, &T::And) {
        expect(input, T::And)?;
        let right = e_eq(input)?;
        let span = left.span.merge(right.span);
        left = Expr {
            node: ExprNode::And(Box::new(left), Box::new(right)),
            span,
        };
    }
    Ok(left)
}

fn e_eq(input: &mut In) -> ModalResult<Expr> {
    let left = e_prefix(input)?;
    if at(*input, &T::Eq) {
        expect(input, T::Eq)?;
        let right = e_prefix(input)?;
        let span = left.span.merge(right.span);
        Ok(Expr {
            node: ExprNode::Eq(Box::new(left), Box::new(right)),
            span,
        })
    } else {
        Ok(left)
    }
}

fn e_prefix(input: &mut In) -> ModalResult<Expr> {
    if at(*input, &T::Not) {
        let s = expect(input, T::Not)?;
        let p = e_prefix(input)?;
        let span = s.merge(p.span);
        return Ok(Expr {
            node: ExprNode::Not(Box::new(p)),
            span,
        });
    }
    if at(*input, &T::KwForall) || at(*input, &T::KwExists) {
        let is_forall = at(*input, &T::KwForall);
        let kw = expect(input, if is_forall { T::KwForall } else { T::KwExists })?;
        let b = binder(input)?;
        expect(input, T::KwSt)?;
        let body = expr(input)?;
        let span = kw.merge(body.span);
        let node = if is_forall {
            ExprNode::Forall(b, Box::new(body))
        } else {
            ExprNode::Exists(b, Box::new(body))
        };
        return Ok(Expr { node, span });
    }
    if at(*input, &T::KwLambda) {
        let kw = expect(input, T::KwLambda)?;
        let b = binder(input)?;
        expect(input, T::KwSt)?;
        let body = expr(input)?;
        let span = kw.merge(body.span);
        return Ok(Expr {
            node: ExprNode::Lambda(b, Box::new(body)),
            span,
        });
    }
    e_arith(input)
}

fn arith_op(k: &T) -> Option<InfixOp> {
    Some(match k {
        T::Plus => InfixOp::Plus,
        T::Minus => InfixOp::Minus,
        T::Star => InfixOp::Star,
        T::Slash => InfixOp::Slash,
        _ => return None,
    })
}

fn e_arith(input: &mut In) -> ModalResult<Expr> {
    let mut left = e_app(input)?;
    while let Some(op) = peek_kind(*input).and_then(arith_op) {
        *input = &input[1..];
        let right = e_app(input)?;
        let span = left.span.merge(right.span);
        left = Expr {
            node: ExprNode::Infix(Box::new(left), op, Box::new(right)),
            span,
        };
    }
    Ok(left)
}

fn e_app(input: &mut In) -> ModalResult<Expr> {
    let head = e_atom(input)?;
    if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let args: Vec<Expr> = separated(0.., expr, comma).parse_next(input)?;
        let close = expect(input, T::RParen)?;
        let span = head.span.merge(close);
        Ok(Expr {
            node: ExprNode::App(Box::new(head), args),
            span,
        })
    } else {
        Ok(head)
    }
}

fn e_atom(input: &mut In) -> ModalResult<Expr> {
    if at(*input, &T::Hole) {
        let s = expect(input, T::Hole)?;
        return Ok(Expr {
            node: ExprNode::Hole,
            span: s,
        });
    }
    // `?name` — a named argument hole (only meaningful in a tactic-inspect step;
    // rejected elsewhere during elaboration).
    if at(*input, &T::Question) {
        let s = expect(input, T::Question)?;
        let name = ident(input)?;
        return Ok(Expr {
            node: ExprNode::NamedHole(name.text),
            span: s.merge(name.span),
        });
    }
    if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let e = expr(input)?;
        expect(input, T::RParen)?;
        return Ok(e);
    }
    if at(*input, &T::KwFalse) {
        let s = expect(input, T::KwFalse)?;
        return Ok(Expr {
            node: ExprNode::False,
            span: s,
        });
    }
    if let Some(Token {
        kind: T::Number(s),
        span,
    }) = input.first()
    {
        let e = Expr {
            node: ExprNode::Num(s.clone()),
            span: *span,
        };
        *input = &input[1..];
        return Ok(e);
    }
    if let Some(t0) = input.first() {
        if let Some(op) = symop_kind(&t0.kind) {
            let span = t0.span;
            *input = &input[1..];
            return Ok(Expr {
                node: ExprNode::Op(op),
                span,
            });
        }
    }
    let q = qname(input)?;
    let span = q.span;
    Ok(Expr {
        node: ExprNode::Var(q),
        span,
    })
}

// ---- sequents, contexts, params -------------------------------------------

fn term_or_proof_binding(input: &mut In) -> ModalResult<FormalParam> {
    let names: Vec<Name> = repeat_min1(input, bind_name)?;
    if at(*input, &T::Colon) {
        expect(input, T::Colon)?;
        let ty = type_expr(input)?;
        Ok(FormalParam::Term(TermBinding { names, ty }))
    } else if at(*input, &T::ColonEq) {
        if names.len() != 1 {
            return backtrack();
        }
        expect(input, T::ColonEq)?;
        let p = expr(input)?;
        Ok(FormalParam::Proof(ProofBinding {
            name: names.into_iter().next().unwrap(),
            prop: p,
        }))
    } else {
        backtrack()
    }
}

/// Context entries up to (but not consuming) the turnstile.
fn context(input: &mut In) -> ModalResult<Vec<ContextEntry>> {
    let mut entries = Vec::new();
    loop {
        if at(*input, &T::Turnstile) {
            break;
        }
        let e = term_or_proof_binding(input)?;
        entries.push(e);
        if at(*input, &T::Comma) || at(*input, &T::Semi) {
            *input = &input[1..];
        } else {
            break;
        }
    }
    Ok(entries)
}

fn sequent(input: &mut In) -> ModalResult<Sequent> {
    let start = cur_span(*input);
    let context = context(input)?;
    expect(input, T::Turnstile)?;
    let p = expr(input)?;
    let span = start.merge(p.span);
    Ok(Sequent {
        context,
        prop: p,
        span,
    })
}

fn formal_params(input: &mut In) -> ModalResult<Vec<FormalParam>> {
    if !at(*input, &T::LParen) {
        return Ok(Vec::new());
    }
    expect(input, T::LParen)?;
    if at(*input, &T::RParen) {
        expect(input, T::RParen)?;
        return Ok(Vec::new());
    }
    let params: Vec<FormalParam> = separated(1.., term_or_proof_binding, comma).parse_next(input)?;
    expect(input, T::RParen)?;
    Ok(params)
}

fn actual_args(input: &mut In) -> ModalResult<Vec<Expr>> {
    expect(input, T::LParen)?;
    let args: Vec<Expr> = separated(0.., expr, comma).parse_next(input)?;
    expect(input, T::RParen)?;
    Ok(args)
}

// ---- proofs ---------------------------------------------------------------

fn kw_by(input: &mut In) -> ModalResult<Span> {
    if at(*input, &T::KwBy) {
        expect(input, T::KwBy)
    } else {
        backtrack()
    }
}

fn proof_ref(input: &mut In) -> ModalResult<ProofRef> {
    let q = qname(input)?;
    let args = if at(*input, &T::LParen) {
        actual_args(input)?
    } else {
        Vec::new()
    };
    let span = q.span;
    Ok(ProofRef {
        name: q,
        args,
        span,
    })
}

fn case_block(input: &mut In) -> ModalResult<CaseBlock> {
    let start = expect(input, T::KwCase)?;
    let ctx = context(input)?;
    expect(input, T::Turnstile)?;
    let goal = expr(input)?;
    semi(input)?;
    let pf = proof_block(input)?;
    let span = start.merge(pf.span);
    Ok(CaseBlock {
        context: ctx,
        goal,
        proof: pf,
        span,
    })
}

/// A single `by` step, parsed before the flat chain is folded into the nested
/// `ProofStmt`/`CaseBlock` representation the elaborator consumes.
enum Seg {
    /// `by ref then <ctx ⊢ goal;>` — single-goal continuation.
    Then {
        reference: ProofRef,
        context: Vec<ContextEntry>,
        goal: Expr,
        by_span: Span,
        end_span: Span,
    },
    /// `by ref;` — closes the goal (0 subgoals). Terminal.
    Zero { reference: ProofRef, span: Span },
    /// `by wip;` — admits the goal. Terminal. `hole` is the `?name` if present.
    Admit { span: Span, hole: Option<String> },
    /// `by ref cases case+ (qed|wip);` — branching. Terminal.
    Cases {
        reference: ProofRef,
        cases: Vec<CaseBlock>,
        cases_close: Close,
        span: Span,
    },
    /// `by ref(…)? [then ?g];` — a tactic-inspect step (argument holes / next
    /// goal). Terminal; admits like `wip`.
    Inspect {
        reference: ProofRef,
        subgoal_name: Option<String>,
        span: Span,
    },
}

/// Does any argument (recursively) contain a `?name` hole?
fn contains_named_hole(args: &[Expr]) -> bool {
    fn go(e: &Expr) -> bool {
        match &e.node {
            ExprNode::NamedHole(_) => true,
            ExprNode::App(h, a) => go(h) || a.iter().any(go),
            ExprNode::Infix(a, _, b)
            | ExprNode::Eq(a, b)
            | ExprNode::And(a, b)
            | ExprNode::Or(a, b)
            | ExprNode::Implies(a, b)
            | ExprNode::Iff(a, b) => go(a) || go(b),
            ExprNode::Not(a) => go(a),
            ExprNode::Lambda(_, b) | ExprNode::Forall(_, b) | ExprNode::Exists(_, b) => go(b),
            ExprNode::Var(_) | ExprNode::Num(_) | ExprNode::Op(_) | ExprNode::False | ExprNode::Hole => {
                false
            }
        }
    }
    args.iter().any(go)
}

/// Parse `[context] ⊢ goal ;` — the subgoal restated after `then`.
fn continuation_goal(input: &mut In) -> ModalResult<(Vec<ContextEntry>, Expr, Span)> {
    let ctx = context(input)?;
    expect(input, T::Turnstile)?;
    let goal = expr(input)?;
    let end = semi(input)?;
    Ok((ctx, goal, end))
}

/// Parse the flat body of a proof block: a chain of `by … then …` steps ending
/// in a terminal `by` (0-goal, `wip`, or `cases`). The trailing block
/// terminator is read by `proof_block`.
fn proof_segments(input: &mut In) -> ModalResult<Vec<Seg>> {
    let mut segs = Vec::new();
    loop {
        let by_span = kw_by(input)?;
        // by wip ; — admit (terminal; `then` is not allowed after it).
        // by wip(?name) ; — admit and report a hole at the current goal.
        if at(*input, &T::KwWip) {
            expect(input, T::KwWip)?;
            let hole = if at(*input, &T::LParen) {
                expect(input, T::LParen)?;
                expect(input, T::Question)?;
                let name = ident(input)?;
                expect(input, T::RParen)?;
                Some(name.text)
            } else {
                None
            };
            let end = semi(input)?;
            segs.push(Seg::Admit {
                span: by_span.merge(end),
                hole,
            });
            return Ok(segs);
        }
        let reference = proof_ref(input)?;
        // Trailing `?` marks an inspect step (`by ref?` / `by ref(args)?`).
        let mut inspect = false;
        if at(*input, &T::Question) {
            expect(input, T::Question)?;
            inspect = true;
        }
        inspect |= contains_named_hole(&reference.args);
        if at(*input, &T::KwThen) {
            expect(input, T::KwThen)?;
            // `then ?name` — a terminal subgoal hole (inspect); otherwise a normal
            // single-goal continuation.
            if at(*input, &T::Question) {
                expect(input, T::Question)?;
                let name = ident(input)?;
                let end = semi(input)?;
                segs.push(Seg::Inspect {
                    reference,
                    subgoal_name: Some(name.text),
                    span: by_span.merge(end),
                });
                return Ok(segs);
            }
            let (context, goal, end_span) = continuation_goal(input)?;
            segs.push(Seg::Then { reference, context, goal, by_span, end_span });
            continue;
        } else if inspect {
            let end = semi(input)?;
            segs.push(Seg::Inspect {
                reference,
                subgoal_name: None,
                span: by_span.merge(end),
            });
            return Ok(segs);
        } else if at(*input, &T::KwCases) {
            // Branching: one nested `case` per subgoal, with its own terminator.
            expect(input, T::KwCases)?;
            let cases = repeat_min1(input, case_block)?;
            let cases_close = if at(*input, &T::KwWip) {
                expect(input, T::KwWip)?;
                Close::Wip
            } else {
                expect(input, T::KwQed)?;
                Close::Qed
            };
            let end = semi(input)?;
            segs.push(Seg::Cases {
                reference,
                cases,
                cases_close,
                span: by_span.merge(end),
            });
            return Ok(segs);
        } else {
            // by ref ; — closes the goal.
            let end = semi(input)?;
            segs.push(Seg::Zero {
                reference,
                span: by_span.merge(end),
            });
            return Ok(segs);
        }
    }
}

/// Fold a parsed chain into the nested `ProofStmt` shape. Each `then` step
/// becomes a one-element `cases` whose single case's proof is the rest of the
/// chain; all synthetic terminators inherit the block's physical `close` (taint
/// is uniform along a linear chain).
fn fold_segments(mut segs: Vec<Seg>, close: Close) -> ProofStmt {
    let terminal = segs.pop().expect("proof body has at least one step");
    let mut current = match terminal {
        Seg::Zero { reference, span } => ProofStmt {
            reference: Some(reference),
            admit: false,
            cases: Vec::new(),
            cases_close: close,
            continuation: Cont::Zero,
            hole: None,
            inspect: false,
            subgoal_name: None,
            span,
        },
        Seg::Admit { span, hole } => ProofStmt {
            reference: None,
            admit: true,
            cases: Vec::new(),
            cases_close: close,
            continuation: Cont::Zero,
            hole,
            inspect: false,
            subgoal_name: None,
            span,
        },
        Seg::Cases {
            reference,
            cases,
            cases_close,
            span,
        } => ProofStmt {
            reference: Some(reference),
            admit: false,
            cases,
            cases_close,
            continuation: Cont::Cases,
            hole: None,
            inspect: false,
            subgoal_name: None,
            span,
        },
        Seg::Inspect {
            reference,
            subgoal_name,
            span,
        } => ProofStmt {
            reference: Some(reference),
            admit: true,
            cases: Vec::new(),
            cases_close: close,
            continuation: Cont::Zero,
            hole: None,
            inspect: true,
            subgoal_name,
            span,
        },
        Seg::Then { .. } => unreachable!("proof chain cannot end in `then`"),
    };
    while let Some(seg) = segs.pop() {
        let Seg::Then {
            reference,
            context,
            goal,
            by_span,
            end_span,
        } = seg
        else {
            unreachable!("only the last proof step is terminal");
        };
        let inner = ProofBlock {
            span: current.span,
            stmt: current,
            close,
        };
        let case_span = by_span.merge(end_span);
        let case = CaseBlock {
            context,
            goal,
            proof: inner,
            span: case_span,
        };
        current = ProofStmt {
            reference: Some(reference),
            admit: false,
            cases: vec![case],
            cases_close: close,
            continuation: Cont::Then,
            hole: None,
            inspect: false,
            subgoal_name: None,
            span: case_span,
        };
    }
    current
}

fn proof_block(input: &mut In) -> ModalResult<ProofBlock> {
    let start = expect(input, T::KwProof)?;
    let segs = proof_segments(input)?;
    let (close, end) = if at(*input, &T::KwWip) {
        (Close::Wip, expect(input, T::KwWip)?)
    } else {
        (Close::Qed, expect(input, T::KwQed)?)
    };
    semi(input)?;
    let stmt = fold_segments(segs, close);
    Ok(ProofBlock {
        stmt,
        close,
        span: start.merge(end),
    })
}

// ---- declarations ---------------------------------------------------------

fn import_decl(input: &mut In) -> ModalResult<ImportDecl> {
    let start = expect(input, T::KwImport)?;
    let module = ident(input)?;
    let items = if at(*input, &T::LParen) {
        expect(input, T::LParen)?;
        let list: Vec<ImportItem> = separated(0.., import_item, comma).parse_next(input)?;
        expect(input, T::RParen)?;
        Some(list)
    } else {
        None
    };
    let end = semi(input)?;
    Ok(ImportDecl {
        module,
        items,
        span: start.merge(end),
    })
}

fn import_item(input: &mut In) -> ModalResult<ImportItem> {
    let name = ident(input)?;
    let alias = if at(*input, &T::KwAs) {
        expect(input, T::KwAs)?;
        Some(ident(input)?)
    } else {
        None
    };
    Ok(ImportItem { name, alias })
}

fn sort_decl(input: &mut In) -> ModalResult<SortDecl> {
    let start = expect(input, T::KwSort)?;
    let bindings: Vec<SortBinding> = separated(1.., sort_binding, comma).parse_next(input)?;
    let end = semi(input)?;
    Ok(SortDecl {
        bindings,
        span: start.merge(end),
    })
}

fn sort_binding(input: &mut In) -> ModalResult<SortBinding> {
    let names: Vec<Name> = repeat_min1(input, ident)?;
    expect(input, T::Colon)?;
    let kind = kind_expr(input)?;
    Ok(SortBinding { names, kind })
}

fn op_symbol(input: &mut In) -> ModalResult<Symbol> {
    if let Some(t) = input.first() {
        if let Some(op) = symop_kind(&t.kind) {
            let span = t.span;
            *input = &input[1..];
            return Ok(Symbol::Op(op, span));
        }
        if let T::Number(s) = &t.kind {
            let span = t.span;
            let n = s.clone();
            *input = &input[1..];
            return Ok(Symbol::Number(n, span));
        }
    }
    let q = qname(input)?;
    Ok(Symbol::Name(q))
}

fn op_decl(input: &mut In) -> ModalResult<OpDecl> {
    let start = expect(input, T::KwOp)?;
    let symbol = op_symbol(input)?;
    expect(input, T::Colon)?;
    // function_sig = [type_expr] "->" type_expr.
    // The domain is parsed without a top-level arrow so the right-associative
    // function arrow does not swallow the domain→codomain `->`.
    let domain = if at(*input, &T::Arrow) {
        None
    } else {
        Some(type_sum(input)?)
    };
    expect(input, T::Arrow)?;
    let codomain = type_expr(input)?;
    let end = semi(input)?;
    Ok(OpDecl {
        symbol,
        sig: FunctionSig { domain, codomain },
        span: start.merge(end),
    })
}

fn axiom_decl(input: &mut In) -> ModalResult<AxiomDecl> {
    let start = expect(input, T::KwAxiom)?;
    let name = ident(input)?;
    let params = formal_params(input)?;
    let seq = sequent(input)?;
    let end = semi(input)?;
    Ok(AxiomDecl {
        name,
        params,
        sequent: seq,
        span: start.merge(end),
    })
}

fn rule_decl(input: &mut In) -> ModalResult<RuleDecl> {
    let start = expect(input, T::KwRule)?;
    let name = ident(input)?;
    let params = formal_params(input)?;
    let premises: Vec<Sequent> = separated(1.., sequent, semi).parse_next(input)?;
    expect(input, T::Separator)?;
    let conclusion = sequent(input)?;
    expect(input, T::KwEnd)?;
    let end = semi(input)?;
    Ok(RuleDecl {
        name,
        params,
        premises,
        conclusion,
        span: start.merge(end),
    })
}

fn lemma_like(input: &mut In, theorem: bool) -> ModalResult<LemmaDecl> {
    let start = if theorem {
        expect(input, T::KwTheorem)?
    } else {
        expect(input, T::KwLemma)?
    };
    let name = ident(input)?;
    let params = formal_params(input)?;
    let seq = sequent(input)?;
    semi(input)?;
    let proof = proof_block(input)?;
    let span = start.merge(proof.span);
    Ok(LemmaDecl {
        name,
        params,
        sequent: seq,
        proof,
        span,
    })
}

fn theory_decl(input: &mut In) -> ModalResult<TheoryDecl> {
    let start = expect(input, T::KwTheory)?;
    let name = ident(input)?;
    let params = formal_params(input)?;
    expect(input, T::KwLaws)?;
    let mut items = Vec::new();
    while !at(*input, &T::KwQed) {
        items.push(theory_item(input)?);
    }
    expect(input, T::KwQed)?;
    let end = semi(input)?;
    Ok(TheoryDecl {
        name,
        params,
        items,
        span: start.merge(end),
    })
}

fn theory_item(input: &mut In) -> ModalResult<TheoryItem> {
    if at(*input, &T::KwInclude) {
        let start = expect(input, T::KwInclude)?;
        let name = ident(input)?;
        let args = actual_args(input)?;
        let end = semi(input)?;
        Ok(TheoryItem::Include(IncludeDecl {
            name,
            args,
            span: start.merge(end),
        }))
    } else if at(*input, &T::KwLaw) {
        let start = expect(input, T::KwLaw)?;
        let name = ident(input)?;
        let params = formal_params(input)?;
        let seq = sequent(input)?;
        let end = semi(input)?;
        Ok(TheoryItem::Law(LawDecl {
            name,
            params,
            sequent: seq,
            span: start.merge(end),
        }))
    } else {
        backtrack()
    }
}

fn model_decl(input: &mut In) -> ModalResult<ModelDecl> {
    let start = expect(input, T::KwModel)?;
    let name = ident(input)?;
    expect(input, T::KwSatisfies)?;
    let theory = ident(input)?;
    let args = actual_args(input)?;
    expect(input, T::KwIff)?;
    expect(input, T::KwProps)?;
    let mut laws = Vec::new();
    while !at(*input, &T::KwQed) && !at(*input, &T::KwWip) {
        laws.push(model_law(input)?);
    }
    let close = if at(*input, &T::KwWip) {
        expect(input, T::KwWip)?;
        Close::Wip
    } else {
        expect(input, T::KwQed)?;
        Close::Qed
    };
    let end = semi(input)?;
    Ok(ModelDecl {
        name,
        theory,
        args,
        laws,
        close,
        span: start.merge(end),
    })
}

fn model_law(input: &mut In) -> ModalResult<ModelLaw> {
    let start = expect(input, T::KwLaw)?;
    let law = qname(input)?;
    semi(input)?;
    let proof = proof_block(input)?;
    let span = start.merge(proof.span);
    Ok(ModelLaw { law, proof, span })
}

fn decl(input: &mut In) -> ModalResult<Decl> {
    match peek_kind(*input) {
        Some(T::KwImport) => import_decl(input).map(Decl::Import),
        Some(T::KwSort) => sort_decl(input).map(Decl::Sort),
        Some(T::KwOp) => op_decl(input).map(Decl::Op),
        Some(T::KwAxiom) => axiom_decl(input).map(Decl::Axiom),
        Some(T::KwRule) => rule_decl(input).map(Decl::Rule),
        Some(T::KwLemma) => lemma_like(input, false).map(Decl::Lemma),
        Some(T::KwTheorem) => lemma_like(input, true).map(Decl::Theorem),
        Some(T::KwTheory) => theory_decl(input).map(Decl::Theory),
        Some(T::KwModel) => model_decl(input).map(Decl::Model),
        _ => backtrack(),
    }
}

fn module(input: &mut In) -> ModalResult<Module> {
    let mut decls = Vec::new();
    while !at(*input, &T::Eof) {
        decls.push(decl(input)?);
    }
    expect(input, T::Eof)?;
    Ok(Module { decls })
}

/// Parse a token stream into a [`Module`].
pub fn parse_module(tokens: &[Token], source: &str) -> Result<Module, Vec<Diagnostic>> {
    match module.parse(tokens) {
        Ok(m) => Ok(m),
        Err(e) => Err(vec![parse_error_to_diag(e, tokens, source)]),
    }
}

fn parse_error_to_diag(e: ParseError<In, ContextError>, tokens: &[Token], _source: &str) -> Diagnostic {
    let offset = e.offset();
    let span = tokens
        .get(offset)
        .or_else(|| tokens.last())
        .map(|t| t.span)
        .unwrap_or_default();
    Diagnostic::error("syntax error: unexpected token").with_span(span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::lexer::lex;

    fn parse(src: &str) -> Module {
        let toks = lex(src).expect("lex");
        parse_module(&toks, src).expect("parse")
    }

    #[test]
    fn parses_sort_and_op() {
        let m = parse("sort Nat : Sort;\nop s : Nat -> Nat;\nop + : Nat * Nat -> Nat;\nop 0 : -> Nat;");
        assert_eq!(m.decls.len(), 4);
    }

    #[test]
    fn parses_axiom_and_rule() {
        let src = "axiom refl(T : Sort, x : T) |- x = x;\n\
                   rule transitivity(T : Sort, x y z : T)\n\
                     |- x = y;\n\
                     |- y = z\n\
                     ------------------------\n\
                     |- x = z\n\
                   end;";
        let m = parse(src);
        assert_eq!(m.decls.len(), 2);
    }

    #[test]
    fn parses_lemma_with_proof() {
        // Exercises both a `cases` branch and a flat `then` continuation.
        let src = "lemma add_zero_right\n  |- forall (n : Nat) st n + 0 = n;\nproof\n  by induction(lambda (k : Nat) st k + 0 = k) cases\n    case\n      |- 0 + 0 = 0;\n    proof\n      by add_zero_left(0);\n    qed;\n    case\n      k : Nat;\n      ih := k + 0 = k;\n      |- s(k) + 0 = s(k);\n    proof\n      by rewrite_r(Nat, k + 0, k, ih, s(k) + 0 = s(_))\n      then |- s(k) + 0 = s(k + 0);\n      by add_succ_left(k, 0);\n    qed;\n  qed;\nqed;";
        let m = parse(src);
        assert_eq!(m.decls.len(), 1);
    }

    #[test]
    fn parses_theory_and_model() {
        let src = "theory Semigroup(S : Sort, * : S * S -> S) laws\n  law associativity(x y z : S)\n    |- *( *(x, y), z ) = *( x, *(y, z) );\nqed;";
        let m = parse(src);
        assert_eq!(m.decls.len(), 1);
    }
}
