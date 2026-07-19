//! Declaration collection, import resolution, and proof elaboration.

use crate::core::name::{Interner, Sym};
use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Param, Step};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::Expr;
use crate::diagnostics::{Diagnostic, Fix, Span};
use crate::elaborate::{
    build_fact_rule, build_rule, ctx_to_param, Elab, IncludeSig, LawSig, Scope, TheorySig,
};
use crate::parse::{self, ast};
use std::collections::HashSet;

/// Resolves a module name to its source text.
pub trait SourceResolver {
    fn resolve(&self, module: &str) -> Result<String, String>;
}

/// The result of elaborating a unit.
pub struct CompiledUnit {
    pub interner: Interner,
    /// Names this unit exports (the proof index keys).
    pub exports: Vec<Sym>,
    /// Proof obligations (lemmas, model laws) elaborated into step trees.
    pub obligations: Vec<Obligation>,
    /// The equational rewrite system used for definitional equality.
    pub rewrite: RewriteSystem,
    /// Non-fatal warnings gathered during elaboration (e.g. a hypothesis named
    /// after a rule parameter). These do not fail verification.
    pub warnings: Vec<Diagnostic>,
}

pub struct Obligation {
    pub label: String,
    pub root: Step,
    /// True if this obligation is in progress (contains an admitted `by wip`).
    pub wip: bool,
}

/// Elaborate a unit from its source. When `check_proofs` is false, only
/// signatures and statements are processed (the `typecheck` path).
pub fn elaborate_unit(
    source: &str,
    module_name: &str,
    resolver: &dyn SourceResolver,
    check_proofs: bool,
) -> Result<CompiledUnit, Vec<Diagnostic>> {
    let module = parse::parse(source)?;
    let mut elab = Elab::new();
    elab.source = source.to_string();
    let mut loaded = HashSet::new();
    // Pass 1: build the full signature (local declarations + imports).
    process_decls(&mut elab, &module.decls, module_name, resolver, true, &mut loaded);
    if elab.diags.iter().any(|d| d.severity == crate::diagnostics::Severity::Error) {
        return Err(elab.diags);
    }

    // The equational rewrite system, derived from the signature's equational
    // facts (used for definitional equality during proof elaboration/checking).
    let rewrite = build_rewrite_system(&elab);

    // Pass 2: elaborate proofs against the complete signature.
    let mut obligations = Vec::new();
    if check_proofs {
        for d in &module.decls {
            match d {
                ast::Decl::Lemma(ld) | ast::Decl::Theorem(ld) => {
                    elaborate_lemma_proof(&mut elab, ld, &rewrite, &mut obligations);
                }
                ast::Decl::Model(md) => {
                    elaborate_model(&mut elab, md, &rewrite, &mut obligations);
                }
                _ => {}
            }
        }
    }
    if elab.diags.iter().any(|d| d.severity == crate::diagnostics::Severity::Error) {
        return Err(elab.diags);
    }
    let exports = elab.sig.exported.clone();
    // Any remaining diagnostics are warnings; they travel with the unit and do
    // not fail verification.
    let warnings = elab.diags;
    Ok(CompiledUnit {
        interner: elab.interner,
        exports,
        obligations,
        rewrite,
        warnings,
    })
}

/// Definitional equality in the checker is **beta reduction only**: operators
/// are inert constants and equational axioms are *not* silently applied as
/// rewrite rules. An equation is used only where the proof explicitly invokes
/// it via the congruence rules `backward`/`forward`. Hence the checker runs
/// against an empty rewrite system (`nf` = beta normal form).
fn build_rewrite_system(_elab: &Elab) -> RewriteSystem {
    RewriteSystem::new()
}

fn process_decls(
    elab: &mut Elab,
    decls: &[ast::Decl],
    module_name: &str,
    resolver: &dyn SourceResolver,
    is_root: bool,
    loaded: &mut HashSet<String>,
) {
    for d in decls {
        match d {
            ast::Decl::Import(imp) => {
                if loaded.contains(&imp.module.text) {
                    continue;
                }
                loaded.insert(imp.module.text.clone());
                match resolver.resolve(&imp.module.text) {
                    Ok(src) => match parse::parse(&src) {
                        Ok(m) => {
                            process_decls(
                                elab,
                                &m.decls,
                                &imp.module.text,
                                resolver,
                                false,
                                loaded,
                            );
                        }
                        Err(mut ds) => elab.diags.append(&mut ds),
                    },
                    Err(e) => elab.err(format!("cannot import `{}`: {e}", imp.module.text), imp.span),
                }
            }
            ast::Decl::Sort(sd) => {
                for b in &sd.bindings {
                    let kind = elab.lower_kind(&b.kind);
                    for n in &b.names {
                        register_const(elab, &n.text, module_name, is_root, kind.clone());
                    }
                }
            }
            ast::Decl::Op(od) => {
                if let Some(ty) = lower_op_sig(elab, od) {
                    let name = symbol_text(&od.symbol);
                    register_const(elab, &name, module_name, is_root, ty);
                }
            }
            ast::Decl::Axiom(ad) => {
                if let Ok(rule) = build_fact_rule(elab, &ad.params, &ad.sequent) {
                    register_tactic(elab, &ad.name.text, module_name, is_root, rule);
                    // Mark as an axiom so it can serve as a definitional rule.
                    let s = elab.interner.intern(&ad.name.text);
                    elab.sig.axioms.insert(s);
                    if !is_root {
                        let qk = elab.interner.intern(&format!("{module_name}.{}", ad.name.text));
                        elab.sig.axioms.insert(qk);
                    }
                }
            }
            ast::Decl::Rule(rd) => {
                if let Ok(rule) = build_rule(elab, rd) {
                    register_tactic(elab, &rd.name.text, module_name, is_root, rule);
                }
            }
            ast::Decl::Lemma(ld) | ast::Decl::Theorem(ld) => {
                if let Ok(rule) = build_fact_rule(elab, &ld.params, &ld.sequent) {
                    register_tactic(elab, &ld.name.text, module_name, is_root, rule);
                }
            }
            ast::Decl::Theory(td) => {
                if let Some(ts) = build_theory(elab, td) {
                    let key = elab.interner.intern(&td.name.text);
                    elab.sig.theories.insert(key, ts.clone());
                    if !is_root {
                        let qk = elab.interner.intern(&format!("{module_name}.{}", td.name.text));
                        elab.sig.theories.insert(qk, ts);
                    } else {
                        elab.sig.exported.push(key);
                    }
                }
            }
            ast::Decl::Model(_) => {}
        }
    }
}

fn register_const(elab: &mut Elab, name: &str, module: &str, is_root: bool, ty: Expr) {
    let s = elab.interner.intern(name);
    elab.sig.consts.insert(s, ty.clone());
    if is_root {
        elab.sig.exported.push(s);
    } else {
        let qk = elab.interner.intern(&format!("{module}.{name}"));
        elab.sig.consts.insert(qk, ty);
    }
}

fn register_tactic(elab: &mut Elab, name: &str, module: &str, is_root: bool, rule: InlinedRule) {
    let s = elab.interner.intern(name);
    elab.sig.tactics.insert(s, rule.clone());
    if is_root {
        elab.sig.exported.push(s);
    } else {
        let qk = elab.interner.intern(&format!("{module}.{name}"));
        elab.sig.tactics.insert(qk, rule);
    }
}

fn symbol_text(s: &ast::Symbol) -> String {
    match s {
        ast::Symbol::Name(q) => q.name.text.clone(),
        ast::Symbol::Number(n, _) => n.clone(),
        ast::Symbol::Op(op, _) => crate::elaborate::infix_or_sym_text(*op).to_string(),
    }
}

/// Lower an operator signature into a function type, treating unbound names as
/// implicit (universally quantified) sort variables.
fn lower_op_sig(elab: &mut Elab, od: &ast::OpDecl) -> Option<Expr> {
    let mut scope = Scope::new();
    // Bind the explicit type parameters (each `forall (… : kind) st` prefix) as
    // free variables so the signature can refer to them. Any unqualified name in
    // the signature that is neither a declared constant nor a bound type
    // parameter is now an "unbound" error (no implicit generalization).
    let mut params: Vec<(Sym, Expr)> = Vec::new();
    for b in &od.type_params {
        let kind = elab.lower_type(&scope, &b.ty).ok()?;
        for name in &b.names {
            let s = elab.interner.intern(&name.text);
            scope.add_free_pub(s, kind.clone());
            params.push((s, kind.clone()));
        }
    }
    let cod = elab.lower_type(&scope, &od.sig.codomain).ok()?;
    let mut ty = match &od.sig.domain {
        Some(dom) => {
            let d = elab.lower_type(&scope, dom).ok()?;
            Expr::Arrow(Box::new(d), Box::new(cod))
        }
        None => cod,
    };
    // Close over the type parameters in reverse to form nested `Forall(kind, …)`,
    // making the operator's type a dependent function over its sort arguments.
    for (s, kind) in params.iter().rev() {
        ty = Expr::Pi(Box::new(kind.clone()), Box::new(crate::core::term::close(&ty, *s)));
    }
    Some(ty)
}

/// Collect unqualified names in a surface type that are not declared constants
/// (implicit sort variables), in first-occurrence order.
fn collect_type_frees(elab: &Elab, t: &ast::Type, out: &mut Vec<String>) {
    match &t.node {
        ast::TypeNode::Name(q) => {
            if q.module.is_none() {
                push_implicit(elab, &q.name.text, out);
            }
        }
        ast::TypeNode::App(q, args) => {
            if q.module.is_none() {
                push_implicit(elab, &q.name.text, out);
            }
            for a in args {
                collect_type_frees(elab, a, out);
            }
        }
        ast::TypeNode::Product(xs) | ast::TypeNode::Sum(xs) => {
            for x in xs {
                collect_type_frees(elab, x, out);
            }
        }
        ast::TypeNode::Arrow(a, b) => {
            collect_type_frees(elab, a, out);
            collect_type_frees(elab, b, out);
        }
        ast::TypeNode::Forall(b, body) => {
            // The binder's own parameters are bound, not implicit; collect frees
            // from the body but drop any that the binder introduces.
            collect_type_frees(elab, &b.ty, out);
            let mut inner = Vec::new();
            collect_type_frees(elab, body, &mut inner);
            let bound: Vec<&str> = b.names.iter().map(|n| n.text.as_str()).collect();
            for name in inner {
                if !bound.contains(&name.as_str()) && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        ast::TypeNode::Sort | ast::TypeNode::Prop => {}
    }
}

fn push_implicit(elab: &Elab, name: &str, out: &mut Vec<String>) {
    // Already a declared constant? then not implicit.
    // We cannot intern here (no &mut), so check by string against known names.
    if elab.const_name_exists(name) {
        return;
    }
    if !out.iter().any(|n| n == name) {
        out.push(name.to_string());
    }
}

// ---- theories -------------------------------------------------------------

fn build_theory(elab: &mut Elab, td: &ast::TheoryDecl) -> Option<TheorySig> {
    let mut scope = Scope::new();
    let param_entries = elab.lower_telescope(&mut scope, &td.params).ok()?;
    let params: Vec<Param> = param_entries.iter().cloned().map(ctx_to_param).collect();
    let mut laws = Vec::new();
    let mut includes = Vec::new();
    for item in &td.items {
        match item {
            ast::TheoryItem::Law(ld) => {
                // A law's rule: its own params + conclusion (no premises).
                let mut lscope = scope.clone();
                let lparams = match elab.lower_telescope(&mut lscope, &ld.params) {
                    Ok(p) => p,
                    Err(_) => return None,
                };
                let ctx = match elab.lower_telescope(&mut lscope, &ld.sequent.context) {
                    Ok(c) => c,
                    Err(_) => return None,
                };
                let concl = match elab.lower_expr(&mut lscope, &ld.sequent.prop) {
                    Ok(c) => c,
                    Err(_) => return None,
                };
                let mut all = lparams;
                all.extend(ctx);
                let name = elab.interner.intern(&ld.name.text);
                laws.push(LawSig {
                    name,
                    rule: InlinedRule {
                        params: all.into_iter().map(ctx_to_param).collect(),
                        premises: Vec::new(),
                        conclusion: concl,
                        is_forall_intro: false,
                        bidirectional: false,
                    },
                });
            }
            ast::TheoryItem::Include(inc) => {
                let theory = elab.interner.intern(&inc.name.text);
                let mut iscope = scope.clone();
                let args: Result<Vec<_>, _> =
                    inc.args.iter().map(|a| elab.lower_expr(&mut iscope, a)).collect();
                if let Ok(args) = args {
                    includes.push(IncludeSig { theory, args });
                }
            }
        }
    }
    Some(TheorySig {
        params,
        laws,
        includes,
    })
}

// ---- lemma proofs ---------------------------------------------------------

fn elaborate_lemma_proof(
    elab: &mut Elab,
    ld: &ast::LemmaDecl,
    rs: &RewriteSystem,
    obligations: &mut Vec<Obligation>,
) {
    let mut scope = Scope::new();
    let params = match elab.lower_telescope(&mut scope, &ld.params) {
        Ok(p) => p,
        Err(_) => return,
    };
    let ctx_entries = match elab.lower_telescope(&mut scope, &ld.sequent.context) {
        Ok(c) => c,
        Err(_) => return,
    };
    let goal = match elab.lower_expr(&mut scope, &ld.sequent.prop) {
        Ok(g) => g,
        Err(_) => return,
    };
    let mut ctx = params;
    ctx.extend(ctx_entries);
    elab.check_goal_welltyped(&ctx, &goal, ld.sequent.prop.span);
    elab.current_proof = Some(ld.name.text.clone());
    let elaborated = elaborate_proof(elab, &ctx, &goal, &ld.proof, ld.span, rs);
    elab.current_proof = None;
    if let Some((root, wip)) = elaborated {
        obligations.push(Obligation {
            label: format!("lemma {}", ld.name.text),
            root,
            wip,
        });
    }
}

// ---- models ---------------------------------------------------------------

fn elaborate_model(
    elab: &mut Elab,
    md: &ast::ModelDecl,
    rs: &RewriteSystem,
    obligations: &mut Vec<Obligation>,
) {
    let theory_key = elab.interner.intern(&md.theory.text);
    let theory = match elab.sig.theories.get(&theory_key) {
        Some(t) => t.clone(),
        None => {
            elab.err(format!("unknown theory `{}`", md.theory.text), md.span);
            return;
        }
    };

    // Collect model free variables (sort variables) from the args.
    let mut free_names = Vec::new();
    for a in &md.args {
        collect_expr_frees(elab, a, &mut free_names);
    }
    let mut scope = Scope::new();
    let mut model_ctx: Vec<CtxEntry> = Vec::new();
    for name in &free_names {
        let s = elab.interner.intern(name);
        scope.add_free_pub(s, Expr::Sort);
        model_ctx.push(CtxEntry::Term { name: s, ty: Expr::Sort });
    }
    // Lower the actual args.
    let args: Vec<Expr> = match md.args.iter().map(|a| elab.lower_expr(&mut scope, a)).collect() {
        Ok(a) => a,
        Err(_) => return,
    };
    if args.len() != theory.params.len() {
        elab.err(
            format!(
                "theory `{}` expects {} arguments, got {}",
                md.theory.text,
                theory.params.len(),
                args.len()
            ),
            md.span,
        );
        return;
    }
    // theory param -> arg substitution.
    let subst: Vec<(Sym, Expr)> = theory
        .params
        .iter()
        .map(|p| p.name())
        .zip(args.iter().cloned())
        .collect();

    // Build the transitive law obligations (flat + includes).
    let mut obligations_by_name: Vec<(Sym, Vec<CtxEntry>, Expr)> = Vec::new();
    collect_theory_laws(elab, &theory, &subst, &model_ctx, &mut obligations_by_name);

    // Match provided model laws against obligations.
    let mut proven: HashSet<Sym> = HashSet::new();
    let mut any_wip = false;
    for ml in &md.laws {
        let law_local = elab.interner.intern(&ml.law.name.text);
        let found = obligations_by_name
            .iter()
            .find(|(n, _, _)| *n == law_local)
            .cloned();
        match found {
            Some((_, ctx, goal)) => {
                if !proven.insert(law_local) {
                    elab.err(format!("law `{}` proven more than once", ml.law.name.text), ml.span);
                }
                if let Some((root, wip)) = elaborate_proof(elab, &ctx, &goal, &ml.proof, ml.span, rs) {
                    any_wip |= wip;
                    obligations.push(Obligation {
                        label: format!("model {} law {}", md.name.text, ml.law.name.text),
                        root,
                        wip,
                    });
                }
            }
            None => {
                elab.err(
                    format!("`{}` is not a law of theory `{}`", ml.law.name.text, md.theory.text),
                    ml.span,
                );
            }
        }
    }
    // The model's `props` terminator must match whether any law is `wip`.
    if md.close.is_wip() != any_wip {
        elab.err(terminator_msg(any_wip, "model"), md.span);
    }
    let unproven: Vec<String> = obligations_by_name
        .iter()
        .filter(|(n, _, _)| !proven.contains(n))
        .map(|(n, _, _)| elab.interner.resolve(*n).to_string())
        .collect();
    for name in unproven {
        elab.err(
            format!("model `{}` does not prove law `{name}`", md.name.text),
            md.span,
        );
    }
}

fn collect_theory_laws(
    elab: &mut Elab,
    theory: &TheorySig,
    subst: &[(Sym, Expr)],
    model_ctx: &[CtxEntry],
    out: &mut Vec<(Sym, Vec<CtxEntry>, Expr)>,
) {
    // Included theories first (transitive).
    for inc in &theory.includes {
        if let Some(inner) = elab.sig.theories.get(&inc.theory).cloned() {
            // Map the included theory's params to the include args, themselves
            // substituted by the outer subst.
            let inner_subst: Vec<(Sym, Expr)> = inner
                .params
                .iter()
                .map(|p| p.name())
                .zip(inc.args.iter().map(|a| crate::core::tactic::subst_all(a, subst)))
                .collect();
            collect_theory_laws(elab, &inner, &inner_subst, model_ctx, out);
        }
    }
    for law in &theory.laws {
        if out.iter().any(|(n, _, _)| *n == law.name) {
            continue;
        }
        // Obligation context = model free vars ++ law params (substituted).
        let mut ctx = model_ctx.to_vec();
        for p in &law.rule.params {
            ctx.push(match p {
                Param::Term { name, ty } => CtxEntry::Term {
                    name: *name,
                    ty: crate::core::tactic::subst_all(ty, subst),
                },
                Param::Proof { name, prop } => CtxEntry::Proof {
                    name: *name,
                    prop: crate::core::tactic::subst_all(prop, subst),
                },
            });
        }
        let goal = crate::core::normalize::nf(&crate::core::tactic::subst_all(&law.rule.conclusion, subst));
        out.push((law.name, ctx, goal));
    }
}

// ---- the proof elaborator -------------------------------------------------

fn scope_from_ctx(ctx: &[CtxEntry]) -> Scope {
    let mut scope = Scope::new();
    for e in ctx {
        match e {
            CtxEntry::Term { name, ty } => scope.add_free_pub(*name, ty.clone()),
            CtxEntry::Proof { name, .. } => scope.add_free_pub(*name, Expr::Prop),
        }
    }
    scope
}

/// Elaborate a proof block (exactly one `by`) into a step tree, returning the
/// step and whether it is `wip`-tainted (transitively contains a `by wip`).
/// Validates that the block's terminator (`qed`/`wip`) matches its taint.
fn elaborate_proof(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    goal: &Expr,
    block: &ast::ProofBlock,
    span: Span,
    rs: &RewriteSystem,
) -> Option<(Step, bool)> {
    let (step, tainted) = elaborate_step(elab, ctx, goal, &block.stmt, span, rs)?;
    if tainted != block.close.is_wip() {
        // Precise fix: swap just the terminator keyword to match the taint.
        let (title, replacement) = if tainted {
            ("Replace `qed` with `wip`", "wip")
        } else {
            ("Replace `wip` with `qed`", "qed")
        };
        let fix = Fix {
            title: title.to_string(),
            replacement: replacement.to_string(),
            span: block.close_span,
        };
        elab.err_with_fixes(terminator_msg(tainted, "proof"), block.span, vec![fix]);
        return None;
    }
    Some((step, tainted))
}

/// The error for a block whose terminator does not match its taint.
fn terminator_msg(tainted: bool, what: &str) -> String {
    if tainted {
        format!("this {what} admits a goal (`by wip`) but is closed with `qed`; use `wip`")
    } else {
        format!("`wip` terminator on a complete {what}; use `qed`")
    }
}

fn elaborate_step(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    goal: &Expr,
    stmt: &ast::ProofStmt,
    _span: Span,
    rs: &RewriteSystem,
) -> Option<(Step, bool)> {
    // Admit: `by wip` closes the goal without a proof (tainted).
    if stmt.admit {
        if stmt.inspect {
            let is_rewrite = stmt
                .reference
                .as_ref()
                .map(|r| is_rewrite_ref(elab, ctx, r))
                .unwrap_or(false);
            if is_rewrite {
                report_rewrite_hole(elab, ctx, goal, stmt, rs);
            } else {
                report_tactic_hole(elab, ctx, goal, stmt, rs);
            }
        } else if let Some(name) = &stmt.hole {
            report_hole(elab, ctx, goal, name, stmt.span, rs);
        }
        return Some((admit_step(elab, ctx, goal, stmt.span), true));
    }
    let reference = stmt.reference.as_ref().expect("non-admit statement has a reference");

    // `rewrite` is a suggestion hole, not a checking tactic — it only makes sense with
    // a trailing `?`. Reaching here means it was written without one.
    if is_rewrite_ref(elab, ctx, reference) {
        elab.err(
            "`rewrite` is a suggestion hole — write `by rewrite(<eq>)?;` (with a trailing `?`) \
             to get a `forward` suggestion"
                .to_string(),
            stmt.span,
        );
        return None;
    }

    // Resolve the tactic.
    let (tactic_name, base_rule) = resolve_tactic(elab, ctx, reference)?;

    // Build arguments aligned with the rule parameters, expanding `_` holes
    // against the (instantiated) parameter type.
    let surface_args = reference.args.clone();
    if surface_args.len() != base_rule.params.len() {
        elab.err(
            format!(
                "tactic `{}` expects {} argument(s), got {}",
                reference.name.name.text,
                base_rule.params.len(),
                surface_args.len()
            ),
            stmt.span,
        );
        return None;
    }
    let mut scope = scope_from_ctx(ctx);
    // Types of the context's term variables, for inferring argument types.
    let frees: std::collections::HashMap<Sym, Expr> = ctx
        .iter()
        .filter_map(|e| match e {
            CtxEntry::Term { name, ty } => Some((*name, ty.clone())),
            CtxEntry::Proof { .. } => None,
        })
        .collect();
    let mut args = Vec::new();
    let mut term_subst: Vec<(Sym, Expr)> = Vec::new();
    for (p, sa) in base_rule.params.iter().zip(&surface_args) {
        match p {
            Param::Term { name, ty } => {
                let e = if expr_has_hole(sa) {
                    lower_hole_lambda(elab, &mut scope, sa, ty, &term_subst)?
                } else {
                    elab.lower_expr(&mut scope, sa).ok()?
                };
                // Check the argument against the parameter's (instantiated) type.
                // `ty` refers to earlier params via free variables, which
                // `term_subst` already binds. With explicit type arguments every
                // term is monomorphic, so both a type mismatch and an outright
                // ill-typed argument are hard errors.
                let expected =
                    crate::core::normalize::nf(&crate::core::tactic::subst_all(ty, &term_subst));
                match elab.infer(&frees, &[], &e) {
                    Ok(got) => {
                        if !crate::core::normalize::defeq(&got, &expected) {
                            elab.err(
                                format!(
                                    "argument has type `{}` but parameter `{}` expects `{}`",
                                    crate::core::display::show(&got, &elab.interner),
                                    elab.interner.resolve(*name),
                                    crate::core::display::show(&expected, &elab.interner),
                                ),
                                sa.span,
                            );
                            return None;
                        }
                    }
                    Err(msg) => {
                        elab.err(format!("ill-typed argument: {msg}"), sa.span);
                        return None;
                    }
                }
                term_subst.push((*name, e.clone()));
                args.push(Arg::Term(e));
            }
            Param::Proof { .. } => {
                let stmt_expr = resolve_proof_term(elab, ctx, &mut scope, sa)?;
                args.push(Arg::Proof(stmt_expr));
            }
        }
    }

    // Rename premise eigenvariables to the user's chosen names (per case), and
    // build the per-application rule.
    let n_prem = base_rule.premises.len();
    let tname = reference.name.name.text.clone();
    match stmt.continuation {
        ast::Cont::Zero => {
            if n_prem != 0 {
                elab.err(
                    format!(
                        "tactic `{tname}` generates {n_prem} subgoal(s); continue with \
                         `then` (one goal) or `cases` ({n_prem} goals)"
                    ),
                    stmt.span,
                );
                return None;
            }
        }
        ast::Cont::Then => {
            if n_prem != 1 {
                let hint = if n_prem == 0 {
                    "close it with `by …;` (drop the `then`)"
                } else {
                    "use `cases`, one branch per goal"
                };
                elab.err(
                    format!(
                        "`then` continues a single goal, but tactic `{tname}` generates \
                         {n_prem} subgoal(s); {hint}"
                    ),
                    stmt.span,
                );
                return None;
            }
        }
        ast::Cont::Cases => {
            if n_prem < 2 {
                elab.err(
                    format!(
                        "`cases` is for branching (two or more goals); tactic `{tname}` \
                         generates {n_prem} subgoal(s) — use `then` for a single goal"
                    ),
                    stmt.span,
                );
                return None;
            }
            if stmt.cases.len() != n_prem {
                elab.err(
                    format!(
                        "tactic `{tname}` generates {n_prem} subgoal(s) but {} case(s) were given",
                        stmt.cases.len()
                    ),
                    stmt.span,
                );
                return None;
            }
        }
    }

    let parent_names: HashSet<Sym> = ctx.iter().map(|e| e.name()).collect();
    let mut new_premises = Vec::new();
    for (prem, case) in base_rule.premises.iter().zip(&stmt.cases) {
        // The user's newly introduced entries (names not in the parent context).
        let user_new: Vec<(Sym, bool)> = case
            .context
            .iter()
            .filter_map(|fp| user_entry(elab, fp))
            .filter(|(s, _)| !parent_names.contains(s))
            .collect();
        // Rule 1: a premise hypothesis whose instantiated proposition is already
        // assumed in the parent context is dischargeable — it need not be
        // reintroduced (the existing assumption satisfies it).
        let dischargeable: Vec<bool> = prem
            .ctx
            .iter()
            .map(|e| match e {
                CtxEntry::Proof { prop, .. } => {
                    let p = crate::core::tactic::subst_all(prop, &term_subst);
                    ctx.iter().any(|pe| {
                        matches!(pe, CtxEntry::Proof { prop: pp, .. } if rs.defeq(pp, &p))
                    })
                }
                CtxEntry::Term { .. } => false,
            })
            .collect();
        let required: Vec<&CtxEntry> = prem
            .ctx
            .iter()
            .zip(&dischargeable)
            .filter(|(_, d)| !**d)
            .map(|(e, _)| e)
            .collect();
        if user_new.len() != required.len() {
            elab.err(
                format!(
                    "case introduces {} variable(s) but the rule premise introduces {}",
                    user_new.len(),
                    required.len()
                ),
                case.span,
            );
            return None;
        }
        // Eigenvariable freshness (spec §4.15 and the elimination rules): a
        // freshly introduced variable must not already occur free in the parent
        // context (otherwise it would capture an existing assumption).
        for (s, _) in &user_new {
            let clashes = ctx.iter().any(|e| match e {
                CtxEntry::Term { ty, .. } => ty.has_free(*s),
                CtxEntry::Proof { prop, .. } => prop.has_free(*s),
            });
            if clashes {
                elab.err(
                    "eigenvariable already occurs free in the context",
                    case.span,
                );
                return None;
            }
        }
        // No implicit weakening: every proof hypothesis carried from the parent
        // context must be restated in this continuation's context. Silently
        // dropping an assumption is not allowed (a dedicated rule for discarding
        // assumptions may be added later).
        let stated: HashSet<Sym> = case
            .context
            .iter()
            .filter_map(|fp| user_entry(elab, fp))
            .map(|(s, _)| s)
            .collect();
        for e in ctx {
            if let CtxEntry::Proof { name, .. } = e {
                if !stated.contains(name) {
                    let nm = elab.interner.resolve(*name).to_string();
                    elab.err(format!("assumption `{nm}` was discarded"), case.span);
                    return None;
                }
            }
        }
        // Build the premise: drop the dischargeable hypotheses (they are not
        // added to the subgoal), then rename the required ones to the user's
        // chosen names.
        let mut renamed = prem.clone();
        renamed.ctx = prem
            .ctx
            .iter()
            .zip(&dischargeable)
            .filter(|(_, d)| !**d)
            .map(|(e, _)| e.clone())
            .collect();
        for (eig, (user, _is_proof)) in required.iter().zip(&user_new) {
            let from = eig.name();
            renamed = rename_sequent(&renamed, from, *user);
        }
        new_premises.push(renamed);
    }

    let rule = InlinedRule {
        params: base_rule.params.clone(),
        premises: new_premises,
        conclusion: base_rule.conclusion.clone(),
        is_forall_intro: base_rule.is_forall_intro,
        bidirectional: base_rule.bidirectional,
    };

    // Compute next goals (and run the soundness checks).
    let next_goals = match crate::core::tactic::apply(&rule, &args, goal, ctx, rs) {
        Ok(g) => g,
        Err(e) => {
            let rendered = e.render(&elab.interner);
            elab.err(format!("tactic `{}`: {rendered}", reference.name.name.text), stmt.span);
            return None;
        }
    };

    // Recurse into cases, tracking whether any sub-proof is `wip`-tainted.
    let mut children = Vec::new();
    let mut child_tainted = false;
    for (ng, case) in next_goals.iter().zip(&stmt.cases) {
        // Validate the user-declared goal matches the generated subgoal.
        let mut cscope = scope_from_ctx(&ng.ctx);
        if let Ok(user_goal) = elab.lower_expr(&mut cscope, &case.goal) {
            if !rs.defeq(&user_goal, &ng.goal) {
                let what = match stmt.continuation {
                    ast::Cont::Then => "`then` goal does not match the remaining subgoal",
                    _ => "case goal does not match the generated subgoal",
                };
                elab.err(what, case.span);
                return None;
            }
        }
        let (child, tainted) = elaborate_proof(elab, &ng.ctx, &ng.goal, &case.proof, case.span, rs)?;
        child_tainted |= tainted;
        children.push(child);
    }

    // For a `cases` branching statement, validate its own terminator. (A `then`
    // continuation shares the enclosing block's terminator, checked in
    // `elaborate_proof`, so it is exempt here.)
    if stmt.continuation == ast::Cont::Cases && stmt.cases_close.is_wip() != child_tainted {
        // Precise fix: swap just the `cases` block's terminator keyword.
        let (title, replacement) = if child_tainted {
            ("Replace `qed` with `wip`", "wip")
        } else {
            ("Replace `wip` with `qed`", "qed")
        };
        let fix = Fix {
            title: title.to_string(),
            replacement: replacement.to_string(),
            span: stmt.cases_close_span,
        };
        elab.err_with_fixes(terminator_msg(child_tainted, "`cases` block"), stmt.span, vec![fix]);
        return None;
    }

    let step = Step {
        context: ctx.to_vec(),
        current_goal: goal.clone(),
        tactic_name,
        tactic: rule,
        args,
        next_goals,
        children,
        admitted: false,
        span: stmt.span,
    };
    Some((step, child_tainted))
}

/// An admitted (`by wip`) leaf step: the goal is assumed, not checked.
/// Render a context (telescope) as `a : T, h := P`, for premise subgoals.
fn render_entry(e: &CtxEntry, names: &crate::core::name::Interner) -> String {
    use crate::core::display::show;
    match e {
        CtxEntry::Term { name, ty } => format!("{} : {}", names.resolve(*name), show(ty, names)),
        CtxEntry::Proof { name, prop } => format!("{} := {}", names.resolve(*name), show(prop, names)),
    }
}

fn show_ctx_ext(entries: &[CtxEntry], names: &crate::core::name::Interner) -> String {
    entries
        .iter()
        .map(|e| render_entry(e, names))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The entries a `then`/case continuation must restate to reach `sq` from a
/// parent context of length `parent_len`: every carried proof assumption plus
/// every entry the tactic introduced. (Parent term variables stay implicit.)
fn continuation_entries(
    parent: &[CtxEntry],
    sq_ctx: &[CtxEntry],
    rs: &RewriteSystem,
    names: &Interner,
) -> Vec<String> {
    use crate::core::display::show;
    // Names currently in scope (for the rule-2 name-collision check) and the
    // propositions currently assumed (for the rule-1 "already present" check).
    let mut used: HashSet<String> = parent
        .iter()
        .map(|e| names.resolve(e.name()).to_string())
        .collect();
    let mut props: Vec<Expr> = parent
        .iter()
        .filter_map(|e| match e {
            CtxEntry::Proof { prop, .. } => Some(prop.clone()),
            CtxEntry::Term { .. } => None,
        })
        .collect();
    // Carried proof assumptions are always restated (no implicit weakening).
    let mut out: Vec<String> = parent
        .iter()
        .filter(|e| matches!(e, CtxEntry::Proof { .. }))
        .map(|e| render_entry(e, names))
        .collect();
    // Newly-introduced entries, subject to the freshness rules.
    for e in &sq_ctx[parent.len().min(sq_ctx.len())..] {
        match e {
            CtxEntry::Proof { name, prop } => {
                // Rule 1: the proposition is already assumed → not reintroduced.
                if props.iter().any(|p| rs.defeq(p, prop)) {
                    continue;
                }
                // Rule 2: the name is taken by a different proposition → freshen
                // it to `<base>0`, `<base>1`, … until unused.
                let base = names.resolve(*name).to_string();
                let mut nm = base.clone();
                if used.contains(&nm) {
                    let mut n = 0u32;
                    loop {
                        nm = format!("{base}{n}");
                        if !used.contains(&nm) {
                            break;
                        }
                        n += 1;
                    }
                }
                out.push(format!("{nm} := {}", show(prop, names)));
                used.insert(nm);
                props.push(prop.clone());
            }
            CtxEntry::Term { name, ty } => {
                out.push(format!("{} : {}", names.resolve(*name), show(ty, names)));
                used.insert(names.resolve(*name).to_string());
            }
        }
    }
    out
}

/// Render a `then` continuation, laid out one context entry per line:
///
/// ```text
/// then
///     h := P;
///     x : T
///     ⊢ <goal>;
/// ```
fn render_then(entries: &[String], goal: &str, body_indent: &str) -> String {
    if entries.is_empty() {
        return format!("then ⊢ {goal};");
    }
    let mut b = String::from("then\n");
    let n = entries.len();
    for (i, e) in entries.iter().enumerate() {
        let sep = if i + 1 < n { ";" } else { "" };
        b.push_str(&format!("{body_indent}{e}{sep}\n"));
    }
    b.push_str(&format!("{body_indent}⊢ {goal};"));
    b
}

/// Like [`render_then`] but for one branch of a `cases` skeleton, laid out one
/// context entry per line under a `case` header.
fn render_case(entries: &[String], goal: &str, body_indent: &str) -> String {
    if entries.is_empty() {
        return format!("case ⊢ {goal};");
    }
    let mut b = String::from("case\n");
    let n = entries.len();
    for (i, e) in entries.iter().enumerate() {
        let sep = if i + 1 < n { ";" } else { "" };
        b.push_str(&format!("{body_indent}{e}{sep}\n"));
    }
    b.push_str(&format!("{body_indent}⊢ {goal};"));
    b
}

/// The leading whitespace of the source line containing `span.start` — used to
/// indent multi-line continuation fixes so they align with the `by` they follow.
fn line_indent(source: &str, start: usize) -> String {
    let ls = source[..start.min(source.len())]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    source[ls..start.min(source.len())]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// Is `reference` the `rewrite` suggestion hole (a module-less `rewrite` not shadowed
/// by a user-defined tactic or local hypothesis, which would take precedence)?
fn is_rewrite_ref(elab: &Elab, ctx: &[CtxEntry], reference: &ast::ProofRef) -> bool {
    if reference.name.module.is_some() || reference.name.name.text != "rewrite" {
        return false;
    }
    if let Some(key) = elab.interner.get("rewrite") {
        let shadowed = elab.sig.tactics.contains_key(&key)
            || ctx
                .iter()
                .any(|e| matches!(e, CtxEntry::Proof { name, .. } if *name == key));
        if shadowed {
            return false;
        }
    }
    true
}

/// Replace every occurrence of `from_nf` in `goal_nf` with `to`, via the existing
/// ground rewrite engine (an empty metavar list means exact structural matching).
fn subst_occurrences(goal_nf: &Expr, from_nf: &Expr, to: &Expr) -> Expr {
    let mut sys = RewriteSystem::new();
    sys.push(from_nf.clone(), to.clone(), Vec::new());
    sys.nf(goal_nf)
}

/// Immediate subexpressions of `e`, for subterm traversal.
fn expr_children(e: &Expr) -> Vec<&Expr> {
    match e {
        Expr::App(f, args) => {
            let mut v = Vec::with_capacity(args.len() + 1);
            v.push(f.as_ref());
            v.extend(args.iter());
            v
        }
        Expr::Lam(a, b)
        | Expr::Forall(a, b)
        | Expr::Pi(a, b)
        | Expr::Exists(a, b)
        | Expr::Arrow(a, b)
        | Expr::Eq(a, b)
        | Expr::And(a, b)
        | Expr::Or(a, b)
        | Expr::Implies(a, b)
        | Expr::Iff(a, b) => vec![a.as_ref(), b.as_ref()],
        Expr::Not(a) => vec![a.as_ref()],
        Expr::Product(xs) | Expr::Sum(xs) => xs.iter().collect(),
        Expr::Bound(_) | Expr::Free(_) | Expr::Const(_) | Expr::Sort | Expr::Prop | Expr::False => {
            Vec::new()
        }
    }
}

/// The metavariable binding from the first (pre-order, leftmost-outermost) subterm
/// of `goal` that matches `pat` (with `metas` as wildcards), or `None`.
fn first_subterm_match(goal: &Expr, pat: &Expr, metas: &[Sym]) -> Option<Vec<(Sym, Expr)>> {
    let mut subst = Vec::new();
    if crate::core::rewrite::match_pattern(pat, goal, metas, &mut subst) {
        return Some(subst);
    }
    for child in expr_children(goal) {
        if let Some(s) = first_subterm_match(child, pat, metas) {
            return Some(s);
        }
    }
    None
}

/// Format a `by forward(...)` call across lines, one argument per line indented two
/// spaces past the `by`. The first line carries no leading indent (it is spliced in
/// at the `by`'s existing column); callers prefix `indent` when embedding it in a
/// message.
fn format_forward_call(indent: &str, arg_strs: &[String]) -> String {
    let arg_indent = format!("{indent}  ");
    let body = arg_strs.join(&format!(",\n{arg_indent}"));
    format!("by forward(\n{arg_indent}{body})")
}

/// Report a `rewrite(<eq>)?` suggestion hole. The argument names an equation
/// `lhs = rhs` — either fully applied (`pop_push(A, a, s)`) or with some/all
/// arguments left as holes (`pop_push` bare, or `pop_push(?A, ?x, ?s)`).
///
/// * With holes, the missing arguments are **inferred** by matching the axiom's
///   left side against a subterm of the goal, and the suggestion is the same
///   `rewrite` step with those arguments filled in (`rewrite(pop_push(A, a, s))?`).
/// * Fully applied, the motive is synthesized automatically — abstracting *every*
///   occurrence of `lhs` in the goal — and the suggestion is the paste-able
///   `by forward(T, lhs, rhs, <eq>, P) then <ctx> ⊢ <newgoal>;` step.
///
/// When nothing applies (no matching subterm, an argument that can't be inferred,
/// a non-equation argument, or `forward` not in scope) a plain diagnostic explains
/// why and offers no fix. The step is admitted, so the proof stays incomplete.
fn report_rewrite_hole(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    goal: &Expr,
    stmt: &ast::ProofStmt,
    rs: &RewriteSystem,
) {
    use crate::core::display::show;
    use crate::core::tactic::{apply, subst_all};

    let reference = stmt.reference.as_ref().expect("inspect step has a reference");
    let mut msg = format!(
        "found rewrite hole in `by rewrite`\n\nGoal:\n  {}\n",
        show(goal, &elab.interner)
    );

    // Exactly one argument: the equation (a proof reference, possibly with holes).
    if reference.args.len() != 1 {
        elab.err(
            format!(
                "`rewrite` expects 1 argument (a proof of an equation `a = b`), got {}",
                reference.args.len()
            ),
            stmt.span,
        );
        return;
    }
    let axiom_arg = &reference.args[0];

    // Split the argument into its head reference and its surface arguments.
    let (qname, surface_args) = match &axiom_arg.node {
        ast::ExprNode::Var(q) => (q.clone(), Vec::new()),
        ast::ExprNode::App(head, a) => match &head.node {
            ast::ExprNode::Var(q) => (q.clone(), a.clone()),
            _ => {
                elab.err("expected a proof reference", axiom_arg.span);
                return;
            }
        },
        _ => {
            elab.err("expected a proof reference", axiom_arg.span);
            return;
        }
    };

    // Resolve the equation's rule (a global axiom/lemma or a local hypothesis).
    let pref = ast::ProofRef {
        name: qname.clone(),
        args: Vec::new(),
        span: axiom_arg.span,
    };
    let (_akey, arule) = match resolve_tactic(elab, ctx, &pref) {
        Some(x) => x,
        None => return,
    };
    if !arule.premises.is_empty() {
        elab.err("a `rewrite` argument must reference a fact (no premises)", axiom_arg.span);
        return;
    }
    match &arule.conclusion {
        Expr::Eq(_, _) => {}
        other => {
            msg.push_str(&format!(
                "\nThe argument proves `{}`, which is not an equation `a = b`.\n",
                show(other, &elab.interner)
            ));
            elab.err(msg.trim_end().to_string(), stmt.span);
            return;
        }
    }

    let mut scope = scope_from_ctx(ctx);
    // Classify each parameter as a supplied value (→ `concrete`) or a hole to infer
    // (→ `metas`). A bare reference leaves every parameter a hole.
    let mut concrete_subst: Vec<(Sym, Expr)> = Vec::new();
    let mut metas: Vec<Sym> = Vec::new();
    let n = arule.params.len();
    if surface_args.is_empty() {
        for p in &arule.params {
            match p {
                Param::Term { name, .. } => metas.push(*name),
                Param::Proof { .. } => {
                    elab.err("`rewrite` cannot infer proof arguments — supply them explicitly", axiom_arg.span);
                    return;
                }
            }
        }
    } else {
        if surface_args.len() != n {
            elab.err(
                format!("`{}` takes {} argument(s), got {}", qname.name.text, n, surface_args.len()),
                axiom_arg.span,
            );
            return;
        }
        for (p, sa) in arule.params.iter().zip(&surface_args) {
            let is_hole = matches!(sa.node, ast::ExprNode::NamedHole(_));
            match p {
                Param::Term { name, .. } => {
                    if is_hole {
                        metas.push(*name);
                    } else {
                        match elab.lower_expr(&mut scope, sa) {
                            Ok(v) => concrete_subst.push((*name, v)),
                            Err(_) => return,
                        }
                    }
                }
                Param::Proof { .. } => {
                    if is_hole {
                        elab.err("`rewrite` cannot infer proof arguments — supply them explicitly", sa.span);
                        return;
                    }
                    let _ = resolve_proof_term(elab, ctx, &mut scope, sa);
                }
            }
        }
    }

    let goal_nf = rs.nf(goal);
    let (lhs_pat, rhs_pat) = match &arule.conclusion {
        Expr::Eq(a, b) => ((**a).clone(), (**b).clone()),
        _ => unreachable!("conclusion checked to be an equation above"),
    };

    // --- Inference mode: solve the holes by matching the axiom's left side. ---
    if !metas.is_empty() {
        if arule.params.iter().any(|p| matches!(p, Param::Proof { .. })) {
            elab.err("`rewrite` can only infer arguments for equations without proof hypotheses", axiom_arg.span);
            return;
        }
        let lhs_inst = rs.nf(&subst_all(&lhs_pat, &concrete_subst));
        let solved = match first_subterm_match(&goal_nf, &lhs_inst, &metas) {
            Some(s) => s,
            None => {
                msg.push_str(&format!(
                    "\nNo subterm of the goal matches the left side `{}` of the equation.\n",
                    show(&lhs_inst, &elab.interner)
                ));
                elab.err(msg.trim_end().to_string(), stmt.span);
                return;
            }
        };
        let full: Vec<(Sym, Expr)> = concrete_subst.iter().chain(solved.iter()).cloned().collect();
        let unsolved: Vec<Sym> = metas.iter().copied().filter(|m| !full.iter().any(|(s, _)| s == m)).collect();
        if !unsolved.is_empty() {
            let names = unsolved
                .iter()
                .map(|s| elab.interner.resolve(*s))
                .collect::<Vec<_>>()
                .join(", ");
            msg.push_str(&format!(
                "\nCould not infer {names} from the goal — supply {} explicitly.\n",
                if unsolved.len() == 1 { "it" } else { "them" }
            ));
            elab.err(msg.trim_end().to_string(), stmt.span);
            return;
        }
        // Spell the now-ground axiom application in parameter order.
        let arg_strs: Vec<String> = arule
            .params
            .iter()
            .filter_map(|p| match p {
                Param::Term { name, .. } => {
                    full.iter().find(|(s, _)| s == name).map(|(_, v)| show(v, &elab.interner))
                }
                Param::Proof { .. } => None,
            })
            .collect();
        let axiom_name = match &qname.module {
            Some(m) => format!("{}.{}", m.text, qname.name.text),
            None => qname.name.text.clone(),
        };
        let ground = format!("{axiom_name}({})", arg_strs.join(", "));
        msg.push_str(&format!(
            "\nInferred `{ground}` from the goal.\n\nContinue with:\n  by rewrite({ground})?;\n"
        ));
        let fix = Fix {
            title: format!("rewrite({ground})?"),
            replacement: format!("by rewrite({ground})?;"),
            span: stmt.span,
        };
        elab.err_with_fixes(msg.trim_end().to_string(), stmt.span, vec![fix]);
        return;
    }

    // --- Ground mode: the equation is fully concrete → suggest the forward step. ---
    let _ = &rhs_pat; // rhs comes from the instantiated statement below.
    let eq_stmt = rs.nf(&subst_all(&arule.conclusion, &concrete_subst));
    let (lhs, rhs) = match &eq_stmt {
        Expr::Eq(a, b) => ((**a).clone(), (**b).clone()),
        _ => unreachable!("instantiated conclusion is still an equation"),
    };

    // The sort of both sides (forward's `T : Sort`).
    let frees: std::collections::HashMap<Sym, Expr> = ctx
        .iter()
        .filter_map(|e| match e {
            CtxEntry::Term { name, ty } => Some((*name, ty.clone())),
            CtxEntry::Proof { .. } => None,
        })
        .collect();
    let t_sort = match elab.infer(&frees, &[], &lhs) {
        Ok(t) => rs.nf(&t),
        Err(_) => {
            msg.push_str(&format!(
                "\nCould not infer the type of `{}`.\n",
                show(&lhs, &elab.interner)
            ));
            elab.err(msg.trim_end().to_string(), stmt.span);
            return;
        }
    };

    // Build the motive by abstracting every occurrence of `lhs` in the goal.
    let lhs_nf = rs.nf(&lhs);
    let v = elab.interner.fresh("x");
    let body = subst_occurrences(&goal_nf, &lhs_nf, &Expr::Free(v));
    if body == goal_nf {
        msg.push_str(&format!(
            "\nNo subterm of the goal matches the left side `{}` of the equation.\n",
            show(&lhs, &elab.interner)
        ));
        elab.err(msg.trim_end().to_string(), stmt.span);
        return;
    }
    let motive = Expr::Lam(
        Box::new(t_sort.clone()),
        Box::new(crate::core::term::close(&body, v)),
    );

    // Look up `forward` and validate the rewrite by applying it for real; the returned
    // sequent is the canonical subgoal we restate in the fix.
    let fwd = match elab
        .interner
        .get("forward")
        .and_then(|k| elab.sig.tactics.get(&k))
    {
        Some(r) => r.clone(),
        None => {
            msg.push_str("\n`forward` is not in scope — add `import core(forward);` to use `rewrite`.\n");
            elab.err(msg.trim_end().to_string(), stmt.span);
            return;
        }
    };
    let args = vec![
        Arg::Term(t_sort.clone()),
        Arg::Term(lhs.clone()),
        Arg::Term(rhs.clone()),
        Arg::Proof(eq_stmt.clone()),
        Arg::Term(motive.clone()),
    ];
    let next = match apply(&fwd, &args, goal, ctx, rs) {
        Ok(n) => n,
        Err(e) => {
            let rendered = e.render(&elab.interner);
            msg.push_str(&format!("\nThe rewrite does not apply: {rendered}\n"));
            elab.err(msg.trim_end().to_string(), stmt.span);
            return;
        }
    };

    // The concrete `by forward(...)` call, one argument per line: `T`, `lhs`, `rhs`,
    // and the motive printed via `show` (all valid surface syntax); the axiom spelled
    // exactly as the user wrote it.
    let axiom_src = elab
        .span_text(axiom_arg.span)
        .map(str::to_string)
        .unwrap_or_else(|| show(&eq_stmt, &elab.interner));
    let indent = line_indent(&elab.source, stmt.span.start);
    let call = format_forward_call(
        &indent,
        &[
            show(&t_sort, &elab.interner),
            show(&lhs, &elab.interner),
            show(&rhs, &elab.interner),
            axiom_src,
            show(&motive, &elab.interner),
        ],
    );

    let mut fixes: Vec<Fix> = Vec::new();
    match next.first() {
        None => {
            // `forward` always leaves one subgoal, so this is defensive.
            msg.push_str(&format!("\nThis rewrite closes the goal:\n{indent}{call};\n"));
            fixes.push(Fix {
                title: "Rewrite with `forward`".to_string(),
                replacement: format!("{call};"),
                span: stmt.span,
            });
        }
        Some(sq) => {
            let entries = continuation_entries(ctx, &sq.ctx, rs, &elab.interner);
            let g = show(&sq.goal, &elab.interner);
            let body_indent = format!("{indent}  ");
            let then_block = render_then(&entries, &g, &body_indent);
            let name = stmt.subgoal_name.clone().unwrap_or_else(|| "goal".into());
            if entries.is_empty() {
                msg.push_str(&format!("\nApplying it leaves:\n  ⊢ {g}\n"));
            } else {
                msg.push_str(&format!("\nApplying it leaves:\n  {} ⊢ {g}\n", entries.join("; ")));
            }
            msg.push_str(&format!(
                "\nContinue with:\n{indent}{call}\n{indent}{then_block}\n{indent}by wip(?{name});\n"
            ));
            fixes.push(Fix {
                title: "Rewrite with `forward`".to_string(),
                replacement: format!("{call}\n{indent}{then_block}\n{indent}by wip(?{name});"),
                span: stmt.span,
            });
        }
    }

    elab.err_with_fixes(msg.trim_end().to_string(), stmt.span, fixes);
}

/// Report a **tactic-inspect** step (`by ref(…)?`, `?name` arguments, or a
/// terminal `then ?name`). Argument holes are solved against the current goal
/// with first-order matching + type inference (no unifier needed), then listed
/// with their types/values alongside the resulting subgoal(s). A fully concrete
/// `by ref(…)?` instead applies the tactic and suggests the next step. The step
/// is admitted, so the proof stays incomplete.
fn report_tactic_hole(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    goal: &Expr,
    stmt: &ast::ProofStmt,
    rs: &RewriteSystem,
) {
    use crate::core::display::show;
    use crate::core::rewrite::match_pattern;
    use crate::core::tactic::{apply, subst_all};

    let reference = stmt.reference.as_ref().expect("inspect step has a reference");
    let (tname_sym, rule) = match resolve_tactic(elab, ctx, reference) {
        Some(r) => r,
        None => return,
    };
    let tname = elab.interner.resolve(tname_sym).to_string();
    let n = rule.params.len();
    // `by ref?` with no argument list means "every parameter is a hole".
    let all_holes = reference.args.is_empty() && n > 0;
    if !all_holes && reference.args.len() != n {
        elab.err(
            format!("tactic `{tname}` expects {n} argument(s), got {}", reference.args.len()),
            stmt.span,
        );
        return;
    }

    // Classify each parameter: a concrete value (→ `subst`/`args`) or a hole.
    let mut scope = scope_from_ctx(ctx);
    let mut subst: Vec<(Sym, Expr)> = Vec::new();
    let mut args: Vec<Arg> = Vec::new();
    let mut holes: Vec<(usize, String)> = Vec::new();
    for (i, p) in rule.params.iter().enumerate() {
        let arg = if all_holes { None } else { Some(&reference.args[i]) };
        let hole_name = match arg {
            None => Some(elab.interner.resolve(param_sym(p)).to_string()),
            Some(a) => match &a.node {
                ast::ExprNode::NamedHole(hn) => Some(hn.clone()),
                _ => None,
            },
        };
        match (p, hole_name) {
            (_, Some(hn)) => {
                holes.push((i, hn));
                args.push(match p {
                    Param::Term { name, .. } => Arg::Term(Expr::Free(*name)),
                    Param::Proof { name, .. } => Arg::Proof(Expr::Free(*name)),
                });
            }
            (Param::Term { name, ty }, None) => {
                let a = arg.unwrap();
                let e = if expr_has_hole(a) {
                    match lower_hole_lambda(elab, &mut scope, a, ty, &subst) {
                        Some(e) => e,
                        None => return,
                    }
                } else {
                    match elab.lower_expr(&mut scope, a) {
                        Ok(e) => e,
                        Err(_) => return,
                    }
                };
                subst.push((*name, e.clone()));
                args.push(Arg::Term(e));
            }
            (Param::Proof { .. }, None) => match resolve_proof_term(elab, ctx, &mut scope, arg.unwrap()) {
                Some(e) => args.push(Arg::Proof(e)),
                None => return,
            },
        }
    }

    let mut msg = format!("found tactic hole in `by {tname}`\n\nGoal:\n  {}\n", show(goal, &elab.interner));

    if holes.is_empty() {
        // The concrete tactic application, spelled as in the source but without
        // the trailing inspect `?` (and its `;`) — the basis of a paste-able
        // continuation fix. The statement span covers `by …?;`, so peel the
        // terminating `;`, then the inspect `?`.
        let concrete_by = elab.span_text(stmt.span).map(|t| {
            let t = t.trim_end();
            let t = t.strip_suffix(';').unwrap_or(t).trim_end();
            let t = t.strip_suffix('?').unwrap_or(t).trim_end();
            t.to_string()
        });
        // Continuation fixes are multi-line; indent their extra lines to match
        // the `by` they follow so the pasted `then`/`cases` lines up.
        let indent = line_indent(&elab.source, stmt.span.start);
        let mut fixes: Vec<Fix> = Vec::new();

        // Fully concrete: apply for real and suggest the next step.
        match apply(&rule, &args, goal, ctx, rs) {
            Ok(next) if next.is_empty() => {
                msg.push_str("\nThis closes the goal — replace the `?` with `;`.\n");
                if let Some(by) = &concrete_by {
                    fixes.push(Fix {
                        title: "Close the goal".to_string(),
                        replacement: format!("{by};"),
                        span: stmt.span,
                    });
                }
            }
            Ok(next) => {
                msg.push_str("\nApplying it leaves:\n");
                for (k, sq) in next.iter().enumerate() {
                    let tag = if next.len() > 1 { format!(" (goal {})", k + 1) } else { String::new() };
                    let ext = show_ctx_ext(&sq.ctx[ctx.len().min(sq.ctx.len())..], &elab.interner);
                    let g = show(&sq.goal, &elab.interner);
                    if ext.is_empty() {
                        msg.push_str(&format!("  ⊢ {g}{tag}\n"));
                    } else {
                        msg.push_str(&format!("  {ext} ⊢ {g}{tag}\n"));
                    }
                }
                if next.len() == 1 {
                    let name = stmt.subgoal_name.clone().unwrap_or_else(|| "goal".into());
                    // Restate the remaining subgoal exactly — carried assumptions
                    // and any eigenvariables/hypotheses the tactic introduced — so
                    // the suggested `then …` is a valid, non-weakening continuation.
                    let sq = &next[0];
                    let entries = continuation_entries(ctx, &sq.ctx, rs, &elab.interner);
                    let g = show(&sq.goal, &elab.interner);
                    let body_indent = format!("{indent}  ");
                    let then_block = render_then(&entries, &g, &body_indent);
                    msg.push_str(&format!(
                        "\nContinue with:\n{indent}{then_block}\n{indent}by wip(?{name});\n"
                    ));
                    if let Some(by) = &concrete_by {
                        fixes.push(Fix {
                            title: "Continue with `then`".to_string(),
                            replacement: format!("{by}\n{indent}{then_block}\n{indent}by wip(?{name});"),
                            span: stmt.span,
                        });
                    }
                } else {
                    // Paste-able `cases` skeleton, one `case` per subgoal, each
                    // branch left as a named hole to fill in.
                    let case_body_indent = format!("{indent}    ");
                    let mut skel = String::from("cases\n");
                    for (k, sq) in next.iter().enumerate() {
                        let entries = continuation_entries(ctx, &sq.ctx, rs, &elab.interner);
                        let g = show(&sq.goal, &elab.interner);
                        let case_hdr = render_case(&entries, &g, &case_body_indent);
                        skel.push_str(&format!(
                            "{indent}  {case_hdr}\n{indent}    by wip(?g{});\n{indent}  wip;\n",
                            k + 1
                        ));
                    }
                    skel.push_str(&format!("{indent}wip;"));
                    msg.push_str(&format!("\nContinue with:\n{indent}{skel}\n"));
                    if let Some(by) = &concrete_by {
                        fixes.push(Fix {
                            title: "Continue with `cases`".to_string(),
                            replacement: format!("{by}\n{indent}{skel}"),
                            span: stmt.span,
                        });
                    }
                }
            }
            Err(e) => {
                let rendered = e.render(&elab.interner);
                elab.err(format!("tactic `{tname}`: {rendered}"), stmt.span);
                return;
            }
        }
        elab.err_with_fixes(msg.trim_end().to_string(), stmt.span, fixes);
        return;
    }

    // Solve the term holes: match the (concrete-instantiated) conclusion against
    // the goal, then recover type parameters from solved values via `infer`.
    let metas: Vec<Sym> = holes
        .iter()
        .filter_map(|(i, _)| match &rule.params[*i] {
            Param::Term { name, .. } => Some(*name),
            Param::Proof { .. } => None,
        })
        .collect();
    let frees: std::collections::HashMap<Sym, Expr> = ctx
        .iter()
        .filter_map(|e| match e {
            CtxEntry::Term { name, ty } => Some((*name, ty.clone())),
            CtxEntry::Proof { .. } => None,
        })
        .collect();
    let mut solved: Vec<(Sym, Expr)> = Vec::new();
    let concl = subst_all(&rule.conclusion, &subst);
    match_pattern(&concl, goal, &metas, &mut solved);
    for _ in 0..=n {
        let mut changed = false;
        let cur: Vec<(Sym, Expr)> = subst.iter().chain(solved.iter()).cloned().collect();
        for (hs, val) in solved.clone() {
            if let Some(Param::Term { ty, .. }) = rule
                .params
                .iter()
                .find(|p| matches!(p, Param::Term { name, .. } if *name == hs))
            {
                if let Ok(t) = elab.infer(&frees, &[], &val) {
                    let ty_i = subst_all(ty, &cur);
                    let before = solved.len();
                    match_pattern(&ty_i, &t, &metas, &mut solved);
                    changed |= solved.len() > before;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let full: Vec<(Sym, Expr)> = subst.iter().chain(solved.iter()).cloned().collect();

    // List each hole with its (solved) type / required proposition.
    msg.push_str("\nHoles:\n");
    for (i, hn) in &holes {
        match &rule.params[*i] {
            Param::Term { name, ty } => {
                let ty_s = show(&subst_all(ty, &full), &elab.interner);
                match solved.iter().find(|(s, _)| s == name) {
                    Some((_, v)) => msg.push_str(&format!("  ?{hn} : {ty_s} = {}\n", show(v, &elab.interner))),
                    None => msg.push_str(&format!("  ?{hn} : {ty_s}\n")),
                }
            }
            Param::Proof { prop, .. } => {
                msg.push_str(&format!(
                    "  ?{hn} : ⊢ {}  (needs a proof)\n",
                    show(&subst_all(prop, &full), &elab.interner)
                ));
            }
        }
    }

    // The resulting subgoal(s), with everything known substituted in.
    msg.push_str("\nSubgoal(s):\n");
    for (k, prem) in rule.premises.iter().enumerate() {
        let gname = stmt.subgoal_name.clone().unwrap_or_else(|| {
            if rule.premises.len() > 1 {
                format!("g{}", k + 1)
            } else {
                "g".into()
            }
        });
        let ext_entries: Vec<CtxEntry> = prem
            .ctx
            .iter()
            .map(|e| match e {
                CtxEntry::Term { name, ty } => CtxEntry::Term { name: *name, ty: rs.nf(&subst_all(ty, &full)) },
                CtxEntry::Proof { name, prop } => CtxEntry::Proof { name: *name, prop: rs.nf(&subst_all(prop, &full)) },
            })
            .collect();
        let ext = show_ctx_ext(&ext_entries, &elab.interner);
        let g = show(&rs.nf(&subst_all(&prem.goal, &full)), &elab.interner);
        if ext.is_empty() {
            msg.push_str(&format!("  ?{gname} : ⊢ {g}\n"));
        } else {
            msg.push_str(&format!("  ?{gname} : {ext} ⊢ {g}\n"));
        }
    }

    // If *every* hole is a term hole that got solved from the goal, offer the
    // fully-applied tactic as a fix: substitute each solved value in parameter
    // order, e.g. `by refl?` with `?T = T`, `?x = a` becomes `by refl(T, a)?;`.
    // The inspect `?` is kept so the next check refines it further (its holes are
    // now empty → "replace the `?` with `;`"). Proof holes can't be auto-filled,
    // so their presence suppresses this fix.
    let fillable = !holes.is_empty()
        && holes.iter().all(|(i, _)| {
            matches!(&rule.params[*i], Param::Term { name, .. } if solved.iter().any(|(s, _)| s == name))
        });
    if fillable {
        let mut arg_strs: Vec<String> = Vec::new();
        for (i, p) in rule.params.iter().enumerate() {
            let is_hole = holes.iter().any(|(hi, _)| *hi == i);
            match (p, &args[i]) {
                (Param::Term { name, .. }, _) if is_hole => {
                    if let Some((_, v)) = solved.iter().find(|(s, _)| s == name) {
                        arg_strs.push(show(v, &elab.interner));
                    }
                }
                (Param::Term { .. }, Arg::Term(e)) => arg_strs.push(show(e, &elab.interner)),
                (Param::Proof { .. }, Arg::Proof(e)) => arg_strs.push(show(e, &elab.interner)),
                _ => {}
            }
        }
        if arg_strs.len() == rule.params.len() {
            let call = format!("{tname}({})?", arg_strs.join(", "));
            let fix = Fix {
                title: call.clone(),
                replacement: format!("by {call};"),
                span: stmt.span,
            };
            elab.err_with_fixes(msg.trim_end().to_string(), stmt.span, vec![fix]);
            return;
        }
    }

    elab.err(msg.trim_end().to_string(), stmt.span);
}

/// The name symbol of a rule parameter.
fn param_sym(p: &Param) -> Sym {
    match p {
        Param::Term { name, .. } | Param::Proof { name, .. } => *name,
    }
}

/// Emit a hole report for `by wip(?name)`: the goal to prove, the context in
/// scope, and candidate tactics/assumptions that might discharge it. The proof
/// is still admitted (the module remains incomplete), but the diagnostic guides
/// the next step of an interactive proof.
fn report_hole(elab: &mut Elab, ctx: &[CtxEntry], goal: &Expr, name: &str, span: Span, rs: &RewriteSystem) {
    use crate::core::display::show;
    let g = show(goal, &elab.interner);
    let mut msg = format!("found hole ?{name} : proof\n\nContext:\n");
    if ctx.is_empty() {
        msg.push_str("  (empty)\n");
    } else {
        for e in ctx {
            match e {
                CtxEntry::Term { name, ty } => {
                    msg.push_str(&format!(
                        "  {} : {}\n",
                        elab.interner.resolve(*name),
                        show(ty, &elab.interner)
                    ));
                }
                CtxEntry::Proof { name, prop } => {
                    msg.push_str(&format!(
                        "  {} := {}\n",
                        elab.interner.resolve(*name),
                        show(prop, &elab.interner)
                    ));
                }
            }
        }
    }
    msg.push_str(&format!("\nGoal:\n  {g}\n\nCandidates:\n"));
    let cands = hole_candidates(elab, ctx, goal, rs);
    if cands.is_empty() {
        msg.push_str("  (none found — try a rewrite or introduce a variable)\n");
    } else {
        for c in &cands {
            msg.push_str(&format!("  {}\n", c.label));
        }
    }
    // Each candidate becomes a fix that seeds the hole with a complete,
    // re-checkable step: `by <name>?;`. The trailing `?` inspects the tactic, so
    // the next check reports the arguments it needs (or that it closes the
    // goal). The fix replaces the whole `by wip(?…);` statement, including its
    // `;`, so the result is syntactically complete.
    let fixes: Vec<Fix> = cands
        .iter()
        .map(|c| Fix {
            title: format!("{}?", c.name),
            replacement: format!("by {}?;", c.name),
            span,
        })
        .collect();
    elab.err_with_fixes(msg.trim_end().to_string(), span, fixes);
}

/// A candidate step for a hole: `name` is the bare identifier to insert (as
/// `by <name>`); `label` is the human-facing description shown in the report.
struct Candidate {
    name: String,
    label: String,
}

/// Candidate tactics/assumptions for a hole's goal: matching local hypotheses,
/// signature facts/rules whose conclusion matches the goal shape, and `refl`
/// when the goal is a reflexive equation.
fn hole_candidates(elab: &Elab, ctx: &[CtxEntry], goal: &Expr, rs: &RewriteSystem) -> Vec<Candidate> {
    use crate::core::rewrite::match_pattern;
    let mut out: Vec<Candidate> = Vec::new();
    // 1. Local proof hypotheses that already prove the goal.
    for e in ctx {
        if let CtxEntry::Proof { name, prop } = e {
            if rs.defeq(prop, goal) {
                let n = elab.interner.resolve(*name).to_string();
                out.push(Candidate {
                    label: format!("{n} (local assumption)"),
                    name: n,
                });
            }
        }
    }
    // 2. Signature facts/rules whose conclusion matches the goal shape. Skip
    //    rules whose conclusion is a bare metavariable (they match every goal),
    //    and dedup qualified/unqualified aliases by their base name.
    let mut facts: Vec<Candidate> = Vec::new();
    let mut rules: Vec<Candidate> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (sym, rule) in &elab.sig.tactics {
        let metas: Vec<Sym> = rule
            .params
            .iter()
            .filter_map(|p| match p {
                Param::Term { name, .. } => Some(*name),
                Param::Proof { .. } => None,
            })
            .collect();
        if let Expr::Free(s) = &rule.conclusion {
            if metas.contains(s) {
                continue; // matches anything — not a useful shape hint
            }
        }
        let mut subst = Vec::new();
        if match_pattern(&rule.conclusion, goal, &metas, &mut subst) {
            let full = elab.interner.resolve(*sym);
            let base = full.rsplit('.').next().unwrap_or(full).to_string();
            // Don't suggest the proof currently being elaborated (circular).
            if elab.current_proof.as_deref() == Some(base.as_str()) {
                continue;
            }
            if seen.insert(base.clone()) {
                if rule.premises.is_empty() {
                    facts.push(Candidate { label: format!("{base} (fact)"), name: base });
                } else {
                    rules.push(Candidate { label: format!("{base} (rule)"), name: base });
                }
            }
        }
    }
    facts.sort_by(|a, b| a.label.cmp(&b.label));
    rules.sort_by(|a, b| a.label.cmp(&b.label));
    out.extend(facts);
    out.extend(rules);
    // 3. Reflexivity, when the two sides coincide up to α/β.
    if let Expr::Eq(a, b) = goal {
        if rs.defeq(a, b) && !out.iter().any(|c| c.name == "refl") {
            out.push(Candidate {
                name: "refl".to_string(),
                label: "refl (both sides are definitionally equal)".to_string(),
            });
        }
    }
    out.truncate(12);
    out
}

fn admit_step(elab: &mut Elab, ctx: &[CtxEntry], goal: &Expr, span: Span) -> Step {
    let name = elab.interner.intern("wip");
    Step {
        context: ctx.to_vec(),
        current_goal: goal.clone(),
        tactic_name: name,
        tactic: InlinedRule {
            params: Vec::new(),
            premises: Vec::new(),
            conclusion: goal.clone(),
            is_forall_intro: false,
            bidirectional: false,
        },
        args: Vec::new(),
        next_goals: Vec::new(),
        children: Vec::new(),
        admitted: true,
        span,
    }
}

/// Whether a surface expression contains a `_` hole.
fn expr_has_hole(e: &ast::Expr) -> bool {
    match &e.node {
        ast::ExprNode::Hole => true,
        // `?name` is a distinct argument hole, not a `_` motive; don't expand it.
        ast::ExprNode::NamedHole(_) => false,
        ast::ExprNode::Var(_) | ast::ExprNode::Num(_) | ast::ExprNode::Op(_) | ast::ExprNode::False => false,
        ast::ExprNode::App(h, args) => expr_has_hole(h) || args.iter().any(expr_has_hole),
        ast::ExprNode::Infix(a, _, b)
        | ast::ExprNode::Eq(a, b)
        | ast::ExprNode::And(a, b)
        | ast::ExprNode::Or(a, b)
        | ast::ExprNode::Implies(a, b)
        | ast::ExprNode::Iff(a, b) => expr_has_hole(a) || expr_has_hole(b),
        ast::ExprNode::Not(a) => expr_has_hole(a),
        ast::ExprNode::Lambda(_, body)
        | ast::ExprNode::Forall(_, body)
        | ast::ExprNode::Exists(_, body) => expr_has_hole(body),
    }
}

/// Replace every `_` hole in a surface expression with a variable `name`.
fn replace_holes(e: &ast::Expr, name: &ast::Name) -> ast::Expr {
    let node = match &e.node {
        ast::ExprNode::Hole => ast::ExprNode::Var(ast::QName {
            module: None,
            name: name.clone(),
            span: e.span,
        }),
        ast::ExprNode::Var(_)
        | ast::ExprNode::Num(_)
        | ast::ExprNode::Op(_)
        | ast::ExprNode::False
        | ast::ExprNode::NamedHole(_) => e.node.clone(),
        ast::ExprNode::App(h, args) => ast::ExprNode::App(
            Box::new(replace_holes(h, name)),
            args.iter().map(|a| replace_holes(a, name)).collect(),
        ),
        ast::ExprNode::Infix(a, op, b) => ast::ExprNode::Infix(
            Box::new(replace_holes(a, name)),
            *op,
            Box::new(replace_holes(b, name)),
        ),
        ast::ExprNode::Eq(a, b) => ast::ExprNode::Eq(Box::new(replace_holes(a, name)), Box::new(replace_holes(b, name))),
        ast::ExprNode::And(a, b) => ast::ExprNode::And(Box::new(replace_holes(a, name)), Box::new(replace_holes(b, name))),
        ast::ExprNode::Or(a, b) => ast::ExprNode::Or(Box::new(replace_holes(a, name)), Box::new(replace_holes(b, name))),
        ast::ExprNode::Implies(a, b) => ast::ExprNode::Implies(Box::new(replace_holes(a, name)), Box::new(replace_holes(b, name))),
        ast::ExprNode::Iff(a, b) => ast::ExprNode::Iff(Box::new(replace_holes(a, name)), Box::new(replace_holes(b, name))),
        ast::ExprNode::Not(a) => ast::ExprNode::Not(Box::new(replace_holes(a, name))),
        ast::ExprNode::Lambda(b, body) => ast::ExprNode::Lambda(b.clone(), Box::new(replace_holes(body, name))),
        ast::ExprNode::Forall(b, body) => ast::ExprNode::Forall(b.clone(), Box::new(replace_holes(body, name))),
        ast::ExprNode::Exists(b, body) => ast::ExprNode::Exists(b.clone(), Box::new(replace_holes(body, name))),
    };
    ast::Expr { node, span: e.span }
}

/// Expand a hole-containing argument `sa` into a unary lambda whose binder type
/// is the domain of the (instantiated) parameter type `param_ty`.
fn lower_hole_lambda(
    elab: &mut Elab,
    scope: &mut Scope,
    sa: &ast::Expr,
    param_ty: &Expr,
    term_subst: &[(Sym, Expr)],
) -> Option<Expr> {
    let ty_inst = crate::core::normalize::nf(&crate::core::tactic::subst_all(param_ty, term_subst));
    let dom = match ty_inst {
        Expr::Arrow(d, _) => *d,
        _ => {
            elab.err("`_` requires a parameter of function type", sa.span);
            return None;
        }
    };
    let v = elab.interner.fresh("x");
    let vname = ast::Name {
        text: elab.interner.resolve(v).to_string(),
        span: sa.span,
    };
    let body_surface = replace_holes(sa, &vname);
    scope.add_free_pub(v, dom.clone());
    let body = elab.lower_expr(scope, &body_surface).ok()?;
    Some(Expr::Lam(Box::new(dom), Box::new(crate::core::term::close(&body, v))))
}

fn user_entry(elab: &mut Elab, fp: &ast::FormalParam) -> Option<(Sym, bool)> {
    match fp {
        ast::FormalParam::Term(tb) => tb
            .names
            .first()
            .map(|n| (elab.interner.intern(&n.text), false)),
        ast::FormalParam::Proof(pb) => Some((elab.interner.intern(&pb.name.text), true)),
    }
}

fn rename_sequent(s: &Sequent, from: Sym, to: Sym) -> Sequent {
    Sequent {
        ctx: s
            .ctx
            .iter()
            .map(|e| match e {
                CtxEntry::Term { name, ty } => CtxEntry::Term {
                    name: if *name == from { to } else { *name },
                    ty: ty.rename_free(from, to),
                },
                CtxEntry::Proof { name, prop } => CtxEntry::Proof {
                    name: if *name == from { to } else { *name },
                    prop: prop.rename_free(from, to),
                },
            })
            .collect(),
        goal: s.goal.rename_free(from, to),
    }
}

/// Resolve a tactic reference (a global axiom/rule/lemma or a local proof
/// hypothesis) to its inlined rule.
fn resolve_tactic(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    r: &ast::ProofRef,
) -> Option<(Sym, InlinedRule)> {
    let key = elab.intern_qname_pub(&r.name);
    // Local hypothesis?
    if r.name.module.is_none() {
        for e in ctx {
            if let CtxEntry::Proof { name, prop } = e {
                if *name == key {
                    return Some((
                        key,
                        InlinedRule {
                            params: Vec::new(),
                            premises: Vec::new(),
                            conclusion: prop.clone(),
                            is_forall_intro: false,
                            bidirectional: false,
                        },
                    ));
                }
            }
        }
    }
    if let Some(rule) = elab.sig.tactics.get(&key) {
        return Some((key, rule.clone()));
    }
    elab.err(format!("unknown proof reference `{}`", r.name.name.text), r.span);
    None
}

/// Resolve a proof-reference argument to the statement it proves.
fn resolve_proof_term(
    elab: &mut Elab,
    ctx: &[CtxEntry],
    scope: &mut Scope,
    e: &ast::Expr,
) -> Option<Expr> {
    // The argument is `ref` or `ref(args...)`.
    let (qname, args) = match &e.node {
        ast::ExprNode::Var(q) => (q.clone(), Vec::new()),
        ast::ExprNode::App(head, args) => match &head.node {
            ast::ExprNode::Var(q) => (q.clone(), args.clone()),
            _ => {
                elab.err("expected a proof reference", e.span);
                return None;
            }
        },
        _ => {
            elab.err("expected a proof reference", e.span);
            return None;
        }
    };
    let pref = ast::ProofRef {
        name: qname.clone(),
        args: Vec::new(),
        span: e.span,
    };
    let (_name, rule) = resolve_tactic(elab, ctx, &pref)?;
    if !rule.premises.is_empty() {
        elab.err("a proof argument must reference a fact (no premises)", e.span);
        return None;
    }
    if args.len() != rule.params.len() {
        elab.err("wrong number of arguments to proof reference", e.span);
        return None;
    }
    let mut subst = Vec::new();
    for (p, sa) in rule.params.iter().zip(&args) {
        match p {
            Param::Term { name, .. } => {
                let v = elab.lower_expr(scope, sa).ok()?;
                subst.push((*name, v));
            }
            Param::Proof { .. } => {
                // Nested proof argument: resolve recursively (rare).
                let _ = resolve_proof_term(elab, ctx, scope, sa)?;
            }
        }
    }
    Some(crate::core::normalize::nf(&crate::core::tactic::subst_all(&rule.conclusion, &subst)))
}

/// Collect unqualified non-constant names appearing in an expression (model
/// free variables), in first-occurrence order.
fn collect_expr_frees(elab: &Elab, e: &ast::Expr, out: &mut Vec<String>) {
    match &e.node {
        ast::ExprNode::Var(q) => {
            if q.module.is_none() {
                push_implicit(elab, &q.name.text, out);
            }
        }
        ast::ExprNode::App(h, args) => {
            collect_expr_frees(elab, h, out);
            for a in args {
                collect_expr_frees(elab, a, out);
            }
        }
        ast::ExprNode::Infix(a, _, b) | ast::ExprNode::Eq(a, b) | ast::ExprNode::And(a, b)
        | ast::ExprNode::Or(a, b) | ast::ExprNode::Implies(a, b) | ast::ExprNode::Iff(a, b) => {
            collect_expr_frees(elab, a, out);
            collect_expr_frees(elab, b, out);
        }
        ast::ExprNode::Not(a) => collect_expr_frees(elab, a, out),
        ast::ExprNode::Lambda(b, body)
        | ast::ExprNode::Forall(b, body)
        | ast::ExprNode::Exists(b, body) => {
            // Binder-bound names are not free; but we conservatively skip the
            // binder names by collecting from the type and body and removing
            // the bound names.
            let mut inner = Vec::new();
            collect_type_frees(elab, &b.ty, &mut inner);
            collect_expr_frees(elab, body, &mut inner);
            let bound: HashSet<&str> = b.names.iter().map(|n| n.text.as_str()).collect();
            for n in inner {
                if !bound.contains(n.as_str()) && !out.contains(&n) {
                    out.push(n);
                }
            }
        }
        ast::ExprNode::Op(_)
        | ast::ExprNode::Num(_)
        | ast::ExprNode::False
        | ast::ExprNode::Hole
        | ast::ExprNode::NamedHole(_) => {}
    }
}

#[cfg(test)]
mod fix_tests {
    //! Structured fix suggestions attached to diagnostics (surfaced as
    //! CodeMirror autocomplete completions by the web editor).
    use super::*;
    use std::path::PathBuf;

    /// Resolves `import`s against the real `algae/stdlib/v1` sources on disk.
    struct StdlibResolver;
    impl SourceResolver for StdlibResolver {
        fn resolve(&self, module: &str) -> Result<String, String> {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../algae/stdlib/v1")
                .join(format!("{module}.alg"));
            std::fs::read_to_string(&p).map_err(|e| e.to_string())
        }
    }

    fn diags(src: &str) -> Vec<Diagnostic> {
        match elaborate_unit(src, "t", &StdlibResolver, true) {
            Ok(u) => u.warnings,
            Err(d) => d,
        }
    }

    #[test]
    fn hypothesis_named_after_param_warns() {
        // `P := P` names the hypothesis after the `P : Prop` parameter. It must
        // still verify (freshened internally), but emit a non-fatal warning.
        let src = "rule r(P : Prop)\n  P := P ⊢ False\n  ────────────────────────\n  ⊢ ¬P\nend;\n";
        let ds = diags(src);
        assert!(
            ds.iter().any(|d| d.severity == crate::diagnostics::Severity::Warning
                && d.message.contains("clashes with a name the rule already introduces")),
            "expected a name-clash warning: {ds:?}"
        );
        assert!(
            ds.iter().all(|d| d.severity != crate::diagnostics::Severity::Error),
            "must not be an error: {ds:?}"
        );
    }

    /// Every fix's span must slice back to valid source (so an editor can apply
    /// it) and be non-empty.
    fn assert_spans_valid(src: &str, ds: &[Diagnostic]) {
        for d in ds {
            for f in &d.fixes {
                assert!(
                    src.get(f.span.start..f.span.end).is_some(),
                    "fix span out of bounds: {:?}",
                    f
                );
            }
        }
    }

    #[test]
    fn wip_on_complete_proof_offers_qed_fix() {
        let src = "sort T : Sort;\nop a : -> T;\naxiom ax |- a = a;\nlemma l\n  |- a = a;\nproof\n  by ax;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement == "qed")
            .expect("expected a `qed` terminator fix");
        // The fix targets exactly the `wip` keyword.
        assert_eq!(&src[fix.span.start..fix.span.end], "wip");
    }

    #[test]
    fn qed_on_admitted_proof_offers_wip_fix() {
        let src = "sort T : Sort;\nop a : -> T;\nlemma l\n  |- a = a;\nproof\n  by wip;\nqed;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement == "wip")
            .expect("expected a `wip` terminator fix");
        assert_eq!(&src[fix.span.start..fix.span.end], "qed");
    }

    #[test]
    fn wip_on_complete_cases_block_offers_qed_fix() {
        // A `cases` block whose branches all close (`qed`) but is itself
        // terminated with `wip` should offer to swap that `wip` for `qed`.
        let src = "import core(and_intro);\nlemma both(A B : Prop, x := A, y := B)\n  |- A /\\ B;\nproof\n  by and_intro(A, B) cases\n    case x := A; y := B; |- A; by x; qed;\n    case x := A; y := B; |- B; by y; qed;\n  wip;\nqed;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement == "qed")
            .expect("cases block should offer a `qed` terminator fix");
        // The fix targets exactly the `cases` block's `wip` keyword.
        assert_eq!(&src[fix.span.start..fix.span.end], "wip");
    }

    #[test]
    fn hole_offers_candidate_fixes() {
        let src = "sort T : Sort;\nop a : -> T;\naxiom ax |- a = a;\nlemma l\n  |- a = a;\nproof\n  by wip(?goal);\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fixes: Vec<&Fix> = ds.iter().flat_map(|d| &d.fixes).collect();
        assert!(!fixes.is_empty(), "hole should offer candidate fixes: {ds:?}");
        // Candidates are inserted as a complete, inspectable `by <name>?;` step.
        assert!(fixes.iter().all(|f| f.replacement.starts_with("by ") && f.replacement.ends_with("?;")));
        assert!(fixes.iter().any(|f| f.replacement == "by refl?;" || f.replacement == "by ax?;"));
    }

    #[test]
    fn tactic_hole_offers_continuation_fix() {
        let src = "import core(symmetry);\nsort T : Sort;\nop a : -> T;\nop b : -> T;\naxiom ab |- a = b;\nlemma flip\n  |- b = a;\nproof\n  by symmetry(T, a, b)?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by symmetry"))
            .expect("tactic hole should offer a continuation fix");
        // The concrete `by …` drops the inspect `?;`; the only `?` left is the
        // fresh `wip(?…)` hole the continuation introduces.
        assert!(
            fix.replacement.starts_with("by symmetry(T, a, b)\n"),
            "continuation should re-spell the concrete tactic, got: {:?}",
            fix.replacement
        );
        assert!(!fix.replacement.contains("?;"), "inspect `?;` must be stripped: {:?}", fix.replacement);
        assert!(fix.replacement.contains("then"));
    }

    #[test]
    fn tactic_hole_with_solved_holes_offers_full_application() {
        // `by refl?` on `a = a` solves both holes (?T = T, ?x = a); the fix
        // should be the fully-applied `by refl(T, a)?;`.
        let src = "import core(refl);\nsort T : Sort;\nop a : -> T;\nlemma l\n  |- a = a;\nproof\n  by refl?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by refl("))
            .expect("solved tactic holes should offer a full application");
        assert_eq!(fix.replacement, "by refl(T, a)?;");
        assert_eq!(fix.title, "refl(T, a)?");
    }

    #[test]
    fn rewrite_hole_offers_forward_suggestion() {
        // `by rewrite(ab)?` on `f(a) = f(a)` should suggest the full `forward`
        // step that rewrites `a → b`, leaving `f(b) = f(b)`.
        let src = "import core(forward);\nsort T : Sort;\nop a : -> T;\nop b : -> T;\nop f : T -> T;\naxiom ab |- a = b;\nlemma l\n  |- f(a) = f(a);\nproof\n  by rewrite(ab)?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by forward("))
            .expect("rewrite hole should offer a `forward` continuation fix");
        // The suggestion drops the inspect `?;`; the only `?` left is the fresh
        // `wip(?…)` hole the continuation introduces.
        assert!(fix.replacement.contains("then"), "should restate the subgoal: {:?}", fix.replacement);
        assert!(fix.replacement.contains("f(b) = f(b)"), "should rewrite a → b: {:?}", fix.replacement);
        assert!(fix.replacement.contains("ab"), "should reuse the axiom as written: {:?}", fix.replacement);
        assert!(fix.replacement.trim_end().ends_with("by wip(?goal);"), "ends in a hole: {:?}", fix.replacement);
    }

    #[test]
    fn rewrite_hole_abstracts_all_occurrences() {
        // Both sides of `a = a` are `a`; `rewrite(ab)` must flip every occurrence,
        // leaving `b = b`.
        let src = "import core(forward);\nsort T : Sort;\nop a : -> T;\nop b : -> T;\naxiom ab |- a = b;\nlemma l\n  |- a = a;\nproof\n  by rewrite(ab)?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by forward("))
            .expect("rewrite hole should offer a `forward` continuation fix");
        assert!(fix.replacement.contains("⊢ b = b"), "all occurrences flip: {:?}", fix.replacement);
    }

    #[test]
    fn rewrite_hole_no_match_offers_no_fix() {
        // The equation's left side `a` does not occur in the goal `c = c`, so no
        // suggestion can be made.
        let src = "import core(forward);\nsort T : Sort;\nop a : -> T;\nop b : -> T;\nop c : -> T;\naxiom ab |- a = b;\nlemma l\n  |- c = c;\nproof\n  by rewrite(ab)?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        assert!(
            ds.iter().flat_map(|d| &d.fixes).next().is_none(),
            "no matching subterm should offer no fix: {ds:?}"
        );
        assert!(
            ds.iter().any(|d| d.message.contains("No subterm of the goal matches")),
            "should explain why: {ds:?}"
        );
    }

    #[test]
    fn rewrite_hole_forward_suggestion_is_multiline() {
        // The forward suggestion breaks each argument onto its own line, indented
        // two spaces past the `by`.
        let src = "import core(forward);\nsort T : Sort;\nop a : -> T;\nop b : -> T;\nop f : T -> T;\naxiom ab |- a = b;\nlemma l\n  |- f(a) = f(a);\nproof\n  by rewrite(ab)?;\nwip;\n";
        let ds = diags(src);
        assert_spans_valid(src, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by forward("))
            .expect("ground rewrite should offer a `forward` fix");
        // `by forward(` then each argument on its own 4-space-indented line (the
        // `by` sits at 2 spaces, arguments two further).
        assert!(
            fix.replacement.starts_with("by forward(\n    T,\n    a,\n    b,\n    ab,\n    λ"),
            "arguments should be one-per-line: {:?}",
            fix.replacement
        );
    }

    #[test]
    fn rewrite_hole_infers_axiom_arguments() {
        // A bare axiom reference: the arguments are solved by matching the axiom's
        // left side `pop(A, push(A, x, s))` against the goal.
        let stack = "import core(forward);\nsort Stack : Sort -> Sort;\nop empty : forall (A : Sort) st -> Stack(A);\nop push : forall (A : Sort) st A * Stack(A) -> Stack(A);\nop pop : forall (A : Sort) st Stack(A) -> Stack(A);\nop top : forall (A : Sort) st Stack(A) -> A;\naxiom pop_push(A : Sort, x : A, s : Stack(A)) |- pop(A, push(A, x, s)) = s;\nlemma one_pop(A : Sort, a b : A)\n  |- top(A, pop(A, push(A, a, push(A, b, empty(A))))) = b;\nproof\n  by rewrite(pop_push)?;\nwip;\n";
        let ds = diags(stack);
        assert_spans_valid(stack, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by rewrite("))
            .expect("bare axiom ref should infer its arguments");
        assert_eq!(
            fix.replacement,
            "by rewrite(pop_push(A, a, push(A, b, empty(A))))?;"
        );
    }

    #[test]
    fn rewrite_hole_infers_from_named_holes() {
        // Explicit `?` holes (some concrete, some inferred) resolve the same way.
        let stack = "import core(forward);\nsort Stack : Sort -> Sort;\nop empty : forall (A : Sort) st -> Stack(A);\nop push : forall (A : Sort) st A * Stack(A) -> Stack(A);\nop pop : forall (A : Sort) st Stack(A) -> Stack(A);\nop top : forall (A : Sort) st Stack(A) -> A;\naxiom pop_push(A : Sort, x : A, s : Stack(A)) |- pop(A, push(A, x, s)) = s;\nlemma one_pop(A : Sort, a b : A)\n  |- top(A, pop(A, push(A, a, push(A, b, empty(A))))) = b;\nproof\n  by rewrite(pop_push(A, ?x, ?s))?;\nwip;\n";
        let ds = diags(stack);
        assert_spans_valid(stack, &ds);
        let fix = ds
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement.starts_with("by rewrite("))
            .expect("named holes should be inferred");
        assert_eq!(
            fix.replacement,
            "by rewrite(pop_push(A, a, push(A, b, empty(A))))?;"
        );
    }
}
