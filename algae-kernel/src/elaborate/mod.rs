//! Elaboration: resolve names/imports, lower the surface syntax to the kernel,
//! kind/type-check, and build the signature. Proof elaboration lives in
//! [`proof`].

pub mod proof;

use crate::core::name::{Interner, Sym};
use crate::core::rule::{InlinedRule, Param};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::{close, Expr};
use crate::diagnostics::{Diagnostic, Fix, Span};
use crate::parse::ast;
use std::collections::HashMap;

/// A theory's signature: its parameters, its (transitive-ready) local laws, and
/// the theories it includes.
#[derive(Clone, Debug)]
pub struct TheorySig {
    pub params: Vec<Param>,
    pub laws: Vec<LawSig>,
    pub includes: Vec<IncludeSig>,
}

#[derive(Clone, Debug)]
pub struct LawSig {
    pub name: Sym,
    /// The law as an inference rule (params + conclusion, no premises).
    pub rule: InlinedRule,
}

#[derive(Clone, Debug)]
pub struct IncludeSig {
    pub theory: Sym,
    pub args: Vec<Expr>,
}

/// The resolved signature of a compilation unit (including imported symbols).
#[derive(Debug, Default)]
pub struct Signature {
    /// Term/sort-level constants → their type (e.g. `Nat : Sort`, `+ : ...`).
    pub consts: HashMap<Sym, Expr>,
    /// Tactics (axioms, rules, lemmas, theorems) → inlined rule.
    pub tactics: HashMap<Sym, InlinedRule>,
    /// Which tactic names are axioms (only these become definitional rewrite
    /// rules — using lemmas would risk circular self-justification).
    pub axioms: std::collections::HashSet<Sym>,
    /// Theories by name.
    pub theories: HashMap<Sym, TheorySig>,
    /// Names exported by *this* unit (not imported), for the proof index.
    pub exported: Vec<Sym>,
}

/// Elaboration context.
pub struct Elab {
    pub interner: Interner,
    pub sig: Signature,
    pub diags: Vec<Diagnostic>,
    /// Name of the lemma/theorem/law whose proof is currently being elaborated,
    /// used to keep a proof from suggesting itself as a hole candidate.
    pub current_proof: Option<String>,
    /// The unit's source text, so diagnostics can slice the original spelling of
    /// a span when building a machine-applicable fix. Empty until set.
    pub source: String,
}

/// Lexical scope used while lowering an expression.
#[derive(Clone, Default)]
pub struct Scope {
    /// de Bruijn binder names, innermost last.
    binders: Vec<Sym>,
    /// Free variables in scope → their type.
    frees: HashMap<Sym, Expr>,
    /// Order of free-variable introduction (for building contexts).
    free_order: Vec<Sym>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope::default()
    }
    fn push_binder(&mut self, s: Sym) {
        self.binders.push(s);
    }
    fn pop_binder(&mut self) {
        self.binders.pop();
    }
    fn add_free(&mut self, s: Sym, ty: Expr) {
        if !self.frees.contains_key(&s) {
            self.free_order.push(s);
        }
        self.frees.insert(s, ty);
    }
    /// Public wrapper for adding a free variable (used by proof elaboration).
    pub fn add_free_pub(&mut self, s: Sym, ty: Expr) {
        self.add_free(s, ty);
    }
    fn binder_index(&self, s: Sym) -> Option<u32> {
        self.binders
            .iter()
            .rposition(|&b| b == s)
            .map(|p| (self.binders.len() - 1 - p) as u32)
    }
}

impl Elab {
    pub fn new() -> Elab {
        Elab {
            interner: Interner::new(),
            sig: Signature::default(),
            diags: Vec::new(),
            current_proof: None,
            source: String::new(),
        }
    }

    /// Return the original source text covered by `span`, if in range.
    pub fn span_text(&self, span: Span) -> Option<&str> {
        self.source.get(span.start..span.end)
    }

    pub fn err(&mut self, msg: impl Into<String>, span: Span) {
        self.diags.push(Diagnostic::error(msg).with_span(span));
    }

    /// Like [`err`](Self::err), but attaches machine-applicable fix suggestions
    /// (surfaced as autocomplete completions by the web editor).
    pub fn err_with_fixes(&mut self, msg: impl Into<String>, span: Span, fixes: Vec<Fix>) {
        self.diags
            .push(Diagnostic::error(msg).with_span(span).with_fixes(fixes));
    }

    /// Whether `name` is a declared constant (no interning).
    pub fn const_name_exists(&self, name: &str) -> bool {
        self.interner
            .get(name)
            .map(|s| self.sig.consts.contains_key(&s))
            .unwrap_or(false)
    }

    pub fn intern_qname_pub(&mut self, q: &ast::QName) -> Sym {
        self.intern_qname(q)
    }

    fn intern_qname(&mut self, q: &ast::QName) -> Sym {
        match &q.module {
            Some(m) => {
                let key = format!("{}.{}", m.text, q.name.text);
                self.interner.intern(&key)
            }
            None => self.interner.intern(&q.name.text),
        }
    }

    // ---- lowering: kinds --------------------------------------------------

    pub fn lower_kind(&mut self, k: &ast::Kind) -> Expr {
        match &k.node {
            ast::KindNode::Sort => Expr::Sort,
            ast::KindNode::Product(ks) => Expr::Product(ks.iter().map(|k| self.lower_kind(k)).collect()),
            ast::KindNode::Arrow(a, b) => {
                Expr::Arrow(Box::new(self.lower_kind(a)), Box::new(self.lower_kind(b)))
            }
        }
    }

    // ---- lowering: types --------------------------------------------------

    pub fn lower_type(&mut self, scope: &Scope, t: &ast::Type) -> Result<Expr, ()> {
        match &t.node {
            ast::TypeNode::Sort => Ok(Expr::Sort),
            ast::TypeNode::Prop => Ok(Expr::Prop),
            ast::TypeNode::Name(q) => self.resolve_name(scope, q),
            ast::TypeNode::App(q, args) => {
                let head = self.resolve_name(scope, q)?;
                let args: Result<Vec<_>, _> = args.iter().map(|a| self.lower_type(scope, a)).collect();
                Ok(Expr::app(head, args?))
            }
            ast::TypeNode::Product(ts) => {
                let xs: Result<Vec<_>, _> = ts.iter().map(|x| self.lower_type(scope, x)).collect();
                Ok(Expr::Product(xs?))
            }
            ast::TypeNode::Sum(ts) => {
                let xs: Result<Vec<_>, _> = ts.iter().map(|x| self.lower_type(scope, x)).collect();
                Ok(Expr::Sum(xs?))
            }
            ast::TypeNode::Arrow(a, b) => Ok(Expr::Arrow(
                Box::new(self.lower_type(scope, a)?),
                Box::new(self.lower_type(scope, b)?),
            )),
        }
    }

    // ---- lowering: expressions -------------------------------------------

    fn resolve_name(&mut self, scope: &Scope, q: &ast::QName) -> Result<Expr, ()> {
        if q.module.is_none() {
            let s = self.interner.intern(&q.name.text);
            if let Some(d) = scope.binder_index(s) {
                return Ok(Expr::Bound(d));
            }
            if scope.frees.contains_key(&s) {
                return Ok(Expr::Free(s));
            }
            if self.sig.consts.contains_key(&s) {
                return Ok(Expr::Const(s));
            }
            self.err(format!("unbound name `{}`", q.name.text), q.span);
            Err(())
        } else {
            let s = self.intern_qname(q);
            if self.sig.consts.contains_key(&s) {
                Ok(Expr::Const(s))
            } else {
                let name = self.interner.resolve(s).to_string();
                self.err(format!("unknown qualified name `{name}`"), q.span);
                Err(())
            }
        }
    }

    pub fn lower_expr(&mut self, scope: &mut Scope, e: &ast::Expr) -> Result<Expr, ()> {
        match &e.node {
            ast::ExprNode::Var(q) => self.resolve_name(scope, q),
            ast::ExprNode::Num(n) => {
                let s = self.interner.intern(n);
                if self.sig.consts.contains_key(&s) {
                    Ok(Expr::Const(s))
                } else {
                    self.err(format!("undeclared numeric symbol `{n}`"), e.span);
                    Err(())
                }
            }
            ast::ExprNode::Op(op) => {
                let q = ast::QName {
                    module: None,
                    name: ast::Name {
                        text: symop_text(*op).to_string(),
                        span: e.span,
                    },
                    span: e.span,
                };
                self.resolve_name(scope, &q)
            }
            ast::ExprNode::App(head, args) => {
                let h = self.lower_expr(scope, head)?;
                let a: Result<Vec<_>, _> = args.iter().map(|x| self.lower_expr(scope, x)).collect();
                Ok(Expr::app(h, a?))
            }
            ast::ExprNode::Infix(l, op, r) => {
                let name = infix_text(*op);
                let s = self.interner.intern(name);
                let head = if let Some(d) = scope.binder_index(s) {
                    Expr::Bound(d)
                } else if scope.frees.contains_key(&s) {
                    Expr::Free(s)
                } else if self.sig.consts.contains_key(&s) {
                    Expr::Const(s)
                } else {
                    self.err(format!("unbound operator `{name}`"), e.span);
                    return Err(());
                };
                let lo = self.lower_expr(scope, l)?;
                let ro = self.lower_expr(scope, r)?;
                Ok(Expr::app(head, vec![lo, ro]))
            }
            ast::ExprNode::Lambda(b, body) => self.lower_binder(scope, b, body, BinderKind::Lam),
            ast::ExprNode::Forall(b, body) => self.lower_binder(scope, b, body, BinderKind::Forall),
            ast::ExprNode::Exists(b, body) => self.lower_binder(scope, b, body, BinderKind::Exists),
            ast::ExprNode::Eq(a, b) => Ok(Expr::Eq(
                Box::new(self.lower_expr(scope, a)?),
                Box::new(self.lower_expr(scope, b)?),
            )),
            ast::ExprNode::And(a, b) => Ok(Expr::And(
                Box::new(self.lower_expr(scope, a)?),
                Box::new(self.lower_expr(scope, b)?),
            )),
            ast::ExprNode::Or(a, b) => Ok(Expr::Or(
                Box::new(self.lower_expr(scope, a)?),
                Box::new(self.lower_expr(scope, b)?),
            )),
            ast::ExprNode::Implies(a, b) => Ok(Expr::Implies(
                Box::new(self.lower_expr(scope, a)?),
                Box::new(self.lower_expr(scope, b)?),
            )),
            ast::ExprNode::Iff(a, b) => Ok(Expr::Iff(
                Box::new(self.lower_expr(scope, a)?),
                Box::new(self.lower_expr(scope, b)?),
            )),
            ast::ExprNode::Not(a) => Ok(Expr::Not(Box::new(self.lower_expr(scope, a)?))),
            ast::ExprNode::False => Ok(Expr::False),
            ast::ExprNode::Hole => {
                self.err(
                    "`_` is only allowed as a predicate argument to a tactic",
                    e.span,
                );
                Err(())
            }
            ast::ExprNode::NamedHole(name) => {
                self.err(
                    format!("`?{name}` is only allowed as an argument to an inspected tactic (`by ref(…)?`)"),
                    e.span,
                );
                Err(())
            }
        }
    }

    fn lower_binder(
        &mut self,
        scope: &mut Scope,
        b: &ast::Binder,
        body: &ast::Expr,
        kind: BinderKind,
    ) -> Result<Expr, ()> {
        let ty = self.lower_type(scope, &b.ty)?;
        // Multi-name binders desugar to nested binders.
        let names: Vec<Sym> = b.names.iter().map(|n| self.interner.intern(&n.text)).collect();
        for &n in &names {
            scope.push_binder(n);
        }
        let lowered_body = self.lower_expr(scope, body);
        for _ in &names {
            scope.pop_binder();
        }
        let mut result = lowered_body?;
        for _ in &names {
            result = match kind {
                BinderKind::Lam => Expr::Lam(Box::new(ty.clone()), Box::new(result)),
                BinderKind::Forall => Expr::Forall(Box::new(ty.clone()), Box::new(result)),
                BinderKind::Exists => Expr::Exists(Box::new(ty.clone()), Box::new(result)),
            };
        }
        Ok(result)
    }

    // ---- telescopes (params / contexts) -----------------------------------

    /// Lower a list of formal parameters / context entries into kernel context
    /// entries, extending `scope` with each as a free variable.
    pub fn lower_telescope(
        &mut self,
        scope: &mut Scope,
        params: &[ast::FormalParam],
    ) -> Result<Vec<CtxEntry>, ()> {
        let mut out = Vec::new();
        for p in params {
            match p {
                ast::FormalParam::Term(tb) => {
                    let ty = self.lower_type(scope, &tb.ty)?;
                    for n in &tb.names {
                        let s = self.interner.intern(&n.text);
                        scope.add_free(s, ty.clone());
                        out.push(CtxEntry::Term { name: s, ty: ty.clone() });
                    }
                }
                ast::FormalParam::Proof(pb) => {
                    let prop = self.lower_expr(scope, &pb.prop)?;
                    let s = self.interner.intern(&pb.name.text);
                    scope.add_free(s, Expr::Prop);
                    out.push(CtxEntry::Proof { name: s, prop });
                }
            }
        }
        Ok(out)
    }

    // ---- type checking ----------------------------------------------------

    /// Infer the type of `e` in the given free-variable context and binder
    /// stack (binder types, innermost last).
    pub fn infer(
        &self,
        frees: &HashMap<Sym, Expr>,
        binders: &[Expr],
        e: &Expr,
    ) -> Result<Expr, String> {
        match e {
            Expr::Sort => Ok(Expr::Sort),
            Expr::Prop => Ok(Expr::Sort),
            Expr::False => Ok(Expr::Prop),
            Expr::Const(s) => self
                .sig
                .consts
                .get(s)
                .cloned()
                .ok_or_else(|| "unknown constant".to_string()),
            Expr::Free(s) => frees
                .get(s)
                .cloned()
                .ok_or_else(|| "unknown free variable".to_string()),
            Expr::Bound(i) => {
                let n = binders.len();
                if (*i as usize) < n {
                    Ok(binders[n - 1 - *i as usize].clone())
                } else {
                    Err("unbound de Bruijn index".to_string())
                }
            }
            Expr::App(f, args) => {
                let ft = self.infer(frees, binders, f)?;
                self.apply_type(frees, binders, ft, args)
            }
            Expr::Lam(ty, body) => {
                let mut b2 = binders.to_vec();
                b2.push((**ty).clone());
                let bt = self.infer(frees, &b2, body)?;
                Ok(Expr::Arrow(ty.clone(), Box::new(bt)))
            }
            Expr::Arrow(_, _) | Expr::Product(_) | Expr::Sum(_) => Ok(Expr::Sort),
            Expr::Eq(a, b) => {
                let ta = self.infer(frees, binders, a)?;
                let tb = self.infer(frees, binders, b)?;
                if crate::core::normalize::defeq(&ta, &tb) {
                    Ok(Expr::Prop)
                } else {
                    Err("equality between terms of different types".to_string())
                }
            }
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Implies(a, b) | Expr::Iff(a, b) => {
                self.expect_prop(frees, binders, a)?;
                self.expect_prop(frees, binders, b)?;
                Ok(Expr::Prop)
            }
            Expr::Not(a) => {
                self.expect_prop(frees, binders, a)?;
                Ok(Expr::Prop)
            }
            Expr::Forall(ty, body) | Expr::Exists(ty, body) => {
                let mut b2 = binders.to_vec();
                b2.push((**ty).clone());
                self.expect_prop(frees, &b2, body)?;
                Ok(Expr::Prop)
            }
        }
    }

    fn expect_prop(&self, frees: &HashMap<Sym, Expr>, binders: &[Expr], e: &Expr) -> Result<(), String> {
        let t = self.infer(frees, binders, e)?;
        if crate::core::normalize::defeq(&t, &Expr::Prop) {
            Ok(())
        } else {
            Err("expected a proposition".to_string())
        }
    }

    fn apply_type(
        &self,
        frees: &HashMap<Sym, Expr>,
        binders: &[Expr],
        ft: Expr,
        args: &[Expr],
    ) -> Result<Expr, String> {
        if args.is_empty() {
            return Ok(ft);
        }
        match ft {
            Expr::Arrow(dom, cod) => {
                let domc = crate::core::normalize::nf(&dom);
                match domc {
                    Expr::Product(ds) if ds.len() == args.len() => {
                        for (a, d) in args.iter().zip(&ds) {
                            self.check(frees, binders, a, d)?;
                        }
                        Ok(*cod)
                    }
                    Expr::Product(ds) if ds.len() < args.len() => {
                        for (a, d) in args.iter().take(ds.len()).zip(&ds) {
                            self.check(frees, binders, a, d)?;
                        }
                        self.apply_type(frees, binders, *cod, &args[ds.len()..])
                    }
                    _ => {
                        self.check(frees, binders, &args[0], &dom)?;
                        self.apply_type(frees, binders, *cod, &args[1..])
                    }
                }
            }
            _ => Err("applying a non-function".to_string()),
        }
    }

    pub fn check(
        &self,
        frees: &HashMap<Sym, Expr>,
        binders: &[Expr],
        e: &Expr,
        expected: &Expr,
    ) -> Result<(), String> {
        let got = self.infer(frees, binders, e)?;
        if crate::core::normalize::defeq(&got, expected) {
            Ok(())
        } else {
            Err("type mismatch".to_string())
        }
    }
}

#[derive(Clone, Copy)]
enum BinderKind {
    Lam,
    Forall,
    Exists,
}

/// Public spelling of a symbolic operator (for op-declaration symbols).
pub fn infix_or_sym_text(op: ast::SymOp) -> &'static str {
    symop_text(op)
}

fn symop_text(op: ast::SymOp) -> &'static str {
    match op {
        ast::SymOp::Plus => "+",
        ast::SymOp::Minus => "-",
        ast::SymOp::Star => "*",
        ast::SymOp::Slash => "/",
        ast::SymOp::EqEq => "==",
        ast::SymOp::Lt => "<",
        ast::SymOp::Gt => ">",
        ast::SymOp::Le => "<=",
        ast::SymOp::Ge => ">=",
    }
}

fn infix_text(op: ast::InfixOp) -> &'static str {
    match op {
        ast::InfixOp::Plus => "+",
        ast::InfixOp::Minus => "-",
        ast::InfixOp::Star => "*",
        ast::InfixOp::Slash => "/",
    }
}

/// Build an inline rule from a name's formal params and a sequent (used for
/// axioms and lemmas/theorems: zero premises, the sequent as conclusion).
pub fn build_fact_rule(
    elab: &mut Elab,
    params: &[ast::FormalParam],
    sequent: &ast::Sequent,
) -> Result<InlinedRule, ()> {
    let mut scope = Scope::new();
    let param_entries = elab.lower_telescope(&mut scope, params)?;
    // Sequent-local context entries become additional params.
    let ctx_entries = elab.lower_telescope(&mut scope, &sequent.context)?;
    let conclusion = elab.lower_expr(&mut scope, &sequent.prop)?;
    let mut all = param_entries;
    all.extend(ctx_entries);
    Ok(InlinedRule {
        params: all.into_iter().map(ctx_to_param).collect(),
        premises: Vec::new(),
        conclusion,
        is_forall_intro: false,
        bidirectional: false,
    })
}

pub fn ctx_to_param(e: CtxEntry) -> Param {
    match e {
        CtxEntry::Term { name, ty } => Param::Term { name, ty },
        CtxEntry::Proof { name, prop } => Param::Proof { name, prop },
    }
}

/// Build an inference rule from a `rule` declaration.
pub fn build_rule(elab: &mut Elab, r: &ast::RuleDecl) -> Result<InlinedRule, ()> {
    let mut scope = Scope::new();
    let param_entries = elab.lower_telescope(&mut scope, &r.params)?;
    // `forall_intro` no longer needs a special side-condition flag: its
    // generalized variable is an eigenvariable in the premise context, so the
    // §4.15 side condition is enforced by eigenvariable freshness.
    let is_gen = false;
    let bidirectional = r.name.text == "backward" || r.name.text == "forward";
    // Premises: each premise's context entries are eigenvariables/hypotheses,
    // lowered as additional free variables in a cloned scope.
    let mut premises = Vec::new();
    for prem in &r.premises {
        let mut pscope = scope.clone();
        let ext = elab.lower_telescope(&mut pscope, &prem.context)?;
        let goal = elab.lower_expr(&mut pscope, &prem.prop)?;
        premises.push(Sequent { ctx: ext, goal });
    }
    // Conclusion (its own context, if any, also becomes params — but stdlib
    // rule conclusions have no context).
    let mut cscope = scope.clone();
    let cctx = elab.lower_telescope(&mut cscope, &r.conclusion.context)?;
    let conclusion = elab.lower_expr(&mut cscope, &r.conclusion.prop)?;
    let mut params: Vec<Param> = param_entries.into_iter().map(ctx_to_param).collect();
    params.extend(cctx.into_iter().map(ctx_to_param));
    Ok(InlinedRule {
        params,
        premises,
        conclusion,
        is_forall_intro: is_gen,
        bidirectional,
    })
}

/// Close a conclusion `forall`-style: helper re-exported for proof.rs.
pub fn close_binder(e: &Expr, s: Sym) -> Expr {
    close(e, s)
}
