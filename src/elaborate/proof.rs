//! Declaration collection, import resolution, and proof elaboration.

use crate::core::name::{Interner, Sym};
use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Param, Step};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::Expr;
use crate::diagnostics::{Diagnostic, Span};
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
    /// Transitive dependencies as (module name, source content hash).
    pub deps: Vec<(String, u128)>,
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
    let mut loaded = HashSet::new();
    // Pass 1: build the full signature (local declarations + imports).
    process_decls(&mut elab, &module.decls, module_name, resolver, true, &mut loaded);
    if !elab.diags.is_empty() {
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
    if !elab.diags.is_empty() {
        return Err(elab.diags);
    }
    let exports = elab.sig.exported.clone();
    let deps = elab.deps.clone();
    Ok(CompiledUnit {
        interner: elab.interner,
        exports,
        obligations,
        rewrite,
        deps,
    })
}

/// Build the equational rewrite system from 0-premise equational facts in the
/// signature (sorted for determinism; degenerate bare-metavariable rules such
/// as reflexivity's `x = x` are skipped).
fn build_rewrite_system(elab: &Elab) -> RewriteSystem {
    let mut rs = RewriteSystem::new();
    let mut entries: Vec<(&Sym, &InlinedRule)> = elab.sig.tactics.iter().collect();
    entries.sort_by_key(|(s, _)| s.0);
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for (key, rule) in entries {
        if !rule.premises.is_empty() {
            continue;
        }
        // Only axioms are definitional rewrite rules; using a lemma here would
        // let a lemma's statement justify its own proof.
        if !elab.sig.axioms.contains(key) {
            continue;
        }
        if let Expr::Eq(l, r) = &rule.conclusion {
            let metas: Vec<Sym> = rule
                .params
                .iter()
                .filter_map(|p| match p {
                    Param::Term { name, .. } => Some(*name),
                    Param::Proof { .. } => None,
                })
                .collect();
            // Skip rules whose LHS is a bare metavariable (matches everything).
            if let Expr::Free(s) = l.as_ref() {
                if metas.contains(s) {
                    continue;
                }
            }
            let key = (format!("{l:?}"), format!("{r:?}"));
            if seen.insert(key) {
                rs.push((**l).clone(), (**r).clone(), metas);
            }
        }
    }
    rs
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
                            elab.deps.push((
                                imp.module.text.clone(),
                                crate::bytecode::hash128(src.as_bytes()),
                            ));
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
    // Collect implicit sort variables from the signature.
    let mut implicit = Vec::new();
    if let Some(dom) = &od.sig.domain {
        collect_type_frees(elab, dom, &mut implicit);
    }
    collect_type_frees(elab, &od.sig.codomain, &mut implicit);
    for name in &implicit {
        let s = elab.interner.intern(name);
        scope_add_free(&mut scope, s, Expr::Sort);
    }
    let cod = elab.lower_type(&scope, &od.sig.codomain).ok()?;
    let ty = match &od.sig.domain {
        Some(dom) => {
            let d = elab.lower_type(&scope, dom).ok()?;
            Expr::Arrow(Box::new(d), Box::new(cod))
        }
        None => cod,
    };
    Some(ty)
}

fn scope_add_free(scope: &mut Scope, s: Sym, ty: Expr) {
    // Scope's fields are private; reuse the public telescope path via a tiny
    // shim: we encode the free var by lowering a dummy. Instead, expose through
    // add_free helper.
    scope.add_free_pub(s, ty);
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
                        is_generalization: false,
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
    if let Some((root, wip)) = elaborate_proof(elab, &ctx, &goal, &ld.proof, ld.span, rs) {
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
    if md.wip != any_wip {
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
    if block.stmts.len() != 1 {
        elab.err("a proof block must contain exactly one `by` statement", block.span);
        return None;
    }
    let stmt = &block.stmts[0];
    let (step, tainted) = elaborate_step(elab, ctx, goal, stmt, span, rs)?;
    if tainted != block.wip {
        elab.err(terminator_msg(tainted, "proof"), block.span);
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
        return Some((admit_step(elab, ctx, goal), true));
    }
    let reference = stmt.reference.as_ref().expect("non-admit statement has a reference");

    // Resolve the tactic.
    let (tactic_name, base_rule) = resolve_tactic(elab, ctx, reference)?;

    // Build arguments aligned with the rule parameters, expanding `_` holes
    // against the (instantiated) parameter type.
    let surface_args = reference.args.clone().unwrap_or_default();
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
    if stmt.cases.len() != n_prem {
        elab.err(
            format!(
                "tactic `{}` generates {n_prem} subgoal(s) but {} case(s) were given",
                reference.name.name.text,
                stmt.cases.len()
            ),
            stmt.span,
        );
        return None;
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
        if user_new.len() != prem.ctx.len() {
            elab.err(
                format!(
                    "case introduces {} variable(s) but the rule premise introduces {}",
                    user_new.len(),
                    prem.ctx.len()
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
        // Build the renaming and apply it to the premise.
        let mut renamed = prem.clone();
        for (eig, (user, _is_proof)) in prem.ctx.iter().zip(&user_new) {
            let from = eig.name();
            renamed = rename_sequent(&renamed, from, *user);
        }
        new_premises.push(renamed);
    }

    let rule = InlinedRule {
        params: base_rule.params.clone(),
        premises: new_premises,
        conclusion: base_rule.conclusion.clone(),
        is_generalization: base_rule.is_generalization,
        bidirectional: base_rule.bidirectional,
    };

    // Compute next goals (and run the soundness checks).
    let next_goals = match crate::core::tactic::apply(&rule, &args, goal, ctx, rs) {
        Ok(g) => g,
        Err(e) => {
            elab.err(format!("tactic `{}`: {e}", reference.name.name.text), stmt.span);
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
                elab.err("case goal does not match the generated subgoal", case.span);
                return None;
            }
        }
        let (child, tainted) = elaborate_proof(elab, &ng.ctx, &ng.goal, &case.proof, case.span, rs)?;
        child_tainted |= tainted;
        children.push(child);
    }

    // For a multi-case (`cases … qed/wip`) statement, validate its terminator.
    if !stmt.cases.is_empty() && stmt.cases_wip != child_tainted {
        elab.err(terminator_msg(child_tainted, "`cases` block"), stmt.span);
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
    };
    Some((step, child_tainted))
}

/// An admitted (`by wip`) leaf step: the goal is assumed, not checked.
fn admit_step(elab: &mut Elab, ctx: &[CtxEntry], goal: &Expr) -> Step {
    let name = elab.interner.intern("wip");
    Step {
        context: ctx.to_vec(),
        current_goal: goal.clone(),
        tactic_name: name,
        tactic: InlinedRule {
            params: Vec::new(),
            premises: Vec::new(),
            conclusion: goal.clone(),
            is_generalization: false,
            bidirectional: false,
        },
        args: Vec::new(),
        next_goals: Vec::new(),
        children: Vec::new(),
        admitted: true,
    }
}

/// Whether a surface expression contains a `_` hole.
fn expr_has_hole(e: &ast::Expr) -> bool {
    match &e.node {
        ast::ExprNode::Hole => true,
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
        ast::ExprNode::Var(_) | ast::ExprNode::Num(_) | ast::ExprNode::Op(_) | ast::ExprNode::False => {
            e.node.clone()
        }
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
                            is_generalization: false,
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
        args: None,
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
        ast::ExprNode::Op(_) | ast::ExprNode::Num(_) | ast::ExprNode::False | ast::ExprNode::Hole => {}
    }
}
