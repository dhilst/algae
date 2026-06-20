//! The `.algo` bytecode: a deterministic binary serialization of a compiled
//! unit (its proof step trees + equational system + proof index), plus a stable
//! content hash for cache invalidation.
//!
//! Hand-rolled (no serde/bincode) so the byte layout is an explicit, stable
//! contract — important for the content hash and cross-run determinism.

use crate::core::name::Sym;
use crate::core::rewrite::RewriteSystem;
use crate::core::rule::{Arg, InlinedRule, Param, Step};
use crate::core::sequent::{CtxEntry, Sequent};
use crate::core::term::Expr;
use crate::elaborate::proof::{CompiledUnit, Obligation};

const MAGIC: &[u8; 4] = b"ALGO";
const FORMAT_VERSION: u16 = 1;
const COMPILER_VERSION: u16 = 1;

/// A stable 128-bit FNV-1a hash (deterministic across runs, unlike the std
/// hasher). Used for cache invalidation.
pub fn hash128(bytes: &[u8]) -> u128 {
    const OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
    const PRIME: u128 = 0x0000000001000000000000000000013b;
    let mut h = OFFSET;
    // Domain separator: format version, so a codec change forces a cache miss.
    h ^= FORMAT_VERSION as u128;
    h = h.wrapping_mul(PRIME);
    for &b in bytes {
        h ^= b as u128;
        h = h.wrapping_mul(PRIME);
    }
    h
}

// ---- writer ---------------------------------------------------------------

#[derive(Default)]
struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u128(&mut self, v: u128) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
    }
    fn sym(&mut self, s: Sym) {
        self.u32(s.0);
    }
}

// ---- reader ---------------------------------------------------------------

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Reader<'a> {
        Reader { buf, pos: 0 }
    }
    fn u8(&mut self) -> Result<u8, String> {
        let v = *self.buf.get(self.pos).ok_or("unexpected end of bytecode")?;
        self.pos += 1;
        Ok(v)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.buf.len() {
            return Err("unexpected end of bytecode".into());
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u128(&mut self) -> Result<u128, String> {
        Ok(u128::from_le_bytes(self.take(16)?.try_into().unwrap()))
    }
    fn str(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        let s = self.take(n)?;
        String::from_utf8(s.to_vec()).map_err(|_| "invalid utf8 in bytecode".to_string())
    }
    fn sym(&mut self) -> Result<Sym, String> {
        Ok(Sym(self.u32()?))
    }
}

// ---- Expr -----------------------------------------------------------------

fn w_expr(w: &mut Writer, e: &Expr) {
    match e {
        Expr::Bound(i) => {
            w.u8(0);
            w.u32(*i);
        }
        Expr::Free(s) => {
            w.u8(1);
            w.sym(*s);
        }
        Expr::Const(s) => {
            w.u8(2);
            w.sym(*s);
        }
        Expr::App(f, args) => {
            w.u8(3);
            w_expr(w, f);
            w_exprs(w, args);
        }
        Expr::Lam(t, b) => {
            w.u8(4);
            w_expr(w, t);
            w_expr(w, b);
        }
        Expr::Arrow(a, b) => {
            w.u8(5);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::Product(xs) => {
            w.u8(6);
            w_exprs(w, xs);
        }
        Expr::Sum(xs) => {
            w.u8(7);
            w_exprs(w, xs);
        }
        Expr::Sort => w.u8(8),
        Expr::Prop => w.u8(9),
        Expr::Eq(a, b) => {
            w.u8(10);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::And(a, b) => {
            w.u8(11);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::Or(a, b) => {
            w.u8(12);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::Implies(a, b) => {
            w.u8(13);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::Iff(a, b) => {
            w.u8(14);
            w_expr(w, a);
            w_expr(w, b);
        }
        Expr::Not(a) => {
            w.u8(15);
            w_expr(w, a);
        }
        Expr::False => w.u8(16),
        Expr::Forall(t, b) => {
            w.u8(17);
            w_expr(w, t);
            w_expr(w, b);
        }
        Expr::Exists(t, b) => {
            w.u8(18);
            w_expr(w, t);
            w_expr(w, b);
        }
    }
}

fn w_exprs(w: &mut Writer, xs: &[Expr]) {
    w.u32(xs.len() as u32);
    for x in xs {
        w_expr(w, x);
    }
}

fn r_expr(r: &mut Reader) -> Result<Expr, String> {
    Ok(match r.u8()? {
        0 => Expr::Bound(r.u32()?),
        1 => Expr::Free(r.sym()?),
        2 => Expr::Const(r.sym()?),
        3 => Expr::App(Box::new(r_expr(r)?), r_exprs(r)?),
        4 => Expr::Lam(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        5 => Expr::Arrow(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        6 => Expr::Product(r_exprs(r)?),
        7 => Expr::Sum(r_exprs(r)?),
        8 => Expr::Sort,
        9 => Expr::Prop,
        10 => Expr::Eq(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        11 => Expr::And(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        12 => Expr::Or(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        13 => Expr::Implies(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        14 => Expr::Iff(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        15 => Expr::Not(Box::new(r_expr(r)?)),
        16 => Expr::False,
        17 => Expr::Forall(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        18 => Expr::Exists(Box::new(r_expr(r)?), Box::new(r_expr(r)?)),
        t => return Err(format!("bad expr tag {t}")),
    })
}

fn r_exprs(r: &mut Reader) -> Result<Vec<Expr>, String> {
    let n = r.u32()? as usize;
    (0..n).map(|_| r_expr(r)).collect()
}

// ---- context / sequents / rules / steps -----------------------------------

fn w_entry(w: &mut Writer, e: &CtxEntry) {
    match e {
        CtxEntry::Term { name, ty } => {
            w.u8(0);
            w.sym(*name);
            w_expr(w, ty);
        }
        CtxEntry::Proof { name, prop } => {
            w.u8(1);
            w.sym(*name);
            w_expr(w, prop);
        }
    }
}

fn r_entry(r: &mut Reader) -> Result<CtxEntry, String> {
    Ok(match r.u8()? {
        0 => CtxEntry::Term {
            name: r.sym()?,
            ty: r_expr(r)?,
        },
        1 => CtxEntry::Proof {
            name: r.sym()?,
            prop: r_expr(r)?,
        },
        t => return Err(format!("bad ctx-entry tag {t}")),
    })
}

fn w_entries(w: &mut Writer, es: &[CtxEntry]) {
    w.u32(es.len() as u32);
    for e in es {
        w_entry(w, e);
    }
}
fn r_entries(r: &mut Reader) -> Result<Vec<CtxEntry>, String> {
    let n = r.u32()? as usize;
    (0..n).map(|_| r_entry(r)).collect()
}

fn w_sequent(w: &mut Writer, s: &Sequent) {
    w_entries(w, &s.ctx);
    w_expr(w, &s.goal);
}
fn r_sequent(r: &mut Reader) -> Result<Sequent, String> {
    Ok(Sequent {
        ctx: r_entries(r)?,
        goal: r_expr(r)?,
    })
}

fn w_param(w: &mut Writer, p: &Param) {
    match p {
        Param::Term { name, ty } => {
            w.u8(0);
            w.sym(*name);
            w_expr(w, ty);
        }
        Param::Proof { name, prop } => {
            w.u8(1);
            w.sym(*name);
            w_expr(w, prop);
        }
    }
}
fn r_param(r: &mut Reader) -> Result<Param, String> {
    Ok(match r.u8()? {
        0 => Param::Term {
            name: r.sym()?,
            ty: r_expr(r)?,
        },
        1 => Param::Proof {
            name: r.sym()?,
            prop: r_expr(r)?,
        },
        t => return Err(format!("bad param tag {t}")),
    })
}

fn w_rule(w: &mut Writer, rule: &InlinedRule) {
    w.u32(rule.params.len() as u32);
    for p in &rule.params {
        w_param(w, p);
    }
    w.u32(rule.premises.len() as u32);
    for s in &rule.premises {
        w_sequent(w, s);
    }
    w_expr(w, &rule.conclusion);
    w.u8(rule.is_generalization as u8);
    w.u8(rule.bidirectional as u8);
}
fn r_rule(r: &mut Reader) -> Result<InlinedRule, String> {
    let np = r.u32()? as usize;
    let params = (0..np).map(|_| r_param(r)).collect::<Result<_, _>>()?;
    let npr = r.u32()? as usize;
    let premises = (0..npr).map(|_| r_sequent(r)).collect::<Result<_, _>>()?;
    let conclusion = r_expr(r)?;
    let is_generalization = r.u8()? != 0;
    let bidirectional = r.u8()? != 0;
    Ok(InlinedRule {
        params,
        premises,
        conclusion,
        is_generalization,
        bidirectional,
    })
}

fn w_arg(w: &mut Writer, a: &Arg) {
    match a {
        Arg::Term(e) => {
            w.u8(0);
            w_expr(w, e);
        }
        Arg::Proof(e) => {
            w.u8(1);
            w_expr(w, e);
        }
    }
}
fn r_arg(r: &mut Reader) -> Result<Arg, String> {
    Ok(match r.u8()? {
        0 => Arg::Term(r_expr(r)?),
        1 => Arg::Proof(r_expr(r)?),
        t => return Err(format!("bad arg tag {t}")),
    })
}

fn w_step(w: &mut Writer, s: &Step) {
    w_entries(w, &s.context);
    w_expr(w, &s.current_goal);
    w.sym(s.tactic_name);
    w_rule(w, &s.tactic);
    w.u32(s.args.len() as u32);
    for a in &s.args {
        w_arg(w, a);
    }
    w.u32(s.next_goals.len() as u32);
    for g in &s.next_goals {
        w_sequent(w, g);
    }
    w.u32(s.children.len() as u32);
    for c in &s.children {
        w_step(w, c);
    }
}
fn r_step(r: &mut Reader) -> Result<Step, String> {
    let context = r_entries(r)?;
    let current_goal = r_expr(r)?;
    let tactic_name = r.sym()?;
    let tactic = r_rule(r)?;
    let na = r.u32()? as usize;
    let args = (0..na).map(|_| r_arg(r)).collect::<Result<_, _>>()?;
    let ng = r.u32()? as usize;
    let next_goals = (0..ng).map(|_| r_sequent(r)).collect::<Result<_, _>>()?;
    let nc = r.u32()? as usize;
    let children = (0..nc).map(|_| r_step(r)).collect::<Result<_, _>>()?;
    Ok(Step {
        context,
        current_goal,
        tactic_name,
        tactic,
        args,
        next_goals,
        children,
    })
}

// ---- the .algo file -------------------------------------------------------

/// A decoded `.algo` file: enough to run the checker without re-elaborating.
pub struct AlgoFile {
    pub source_hash: u128,
    /// (module name, source hash) of each transitive dependency.
    pub deps: Vec<(String, u128)>,
    pub strings: Vec<String>,
    pub rewrite: RewriteSystem,
    pub exports: Vec<Sym>,
    pub obligations: Vec<Obligation>,
}

/// Serialize a compiled unit to bytecode.
pub fn encode(
    unit: &CompiledUnit,
    source_hash: u128,
    deps: &[(String, u128)],
) -> Vec<u8> {
    let mut w = Writer::default();
    w.buf.extend_from_slice(MAGIC);
    w.u16(FORMAT_VERSION);
    w.u16(COMPILER_VERSION);
    w.u128(source_hash);
    // Dependencies (sorted by name for determinism).
    let mut deps = deps.to_vec();
    deps.sort_by(|a, b| a.0.cmp(&b.0));
    w.u32(deps.len() as u32);
    for (name, h) in &deps {
        w.str(name);
        w.u128(*h);
    }
    // String table (first-use order).
    let strings = unit.interner.strings();
    w.u32(strings.len() as u32);
    for s in strings {
        w.str(s);
    }
    // Rewrite system.
    w.u32(unit.rewrite.rules.len() as u32);
    for (l, r, metas) in &unit.rewrite.rules {
        w_expr(&mut w, l);
        w_expr(&mut w, r);
        w.u32(metas.len() as u32);
        for m in metas {
            w.sym(*m);
        }
    }
    // Exports (proof index keys).
    w.u32(unit.exports.len() as u32);
    for s in &unit.exports {
        w.sym(*s);
    }
    // Obligations.
    w.u32(unit.obligations.len() as u32);
    for ob in &unit.obligations {
        w.str(&ob.label);
        w_step(&mut w, &ob.root);
    }
    w.buf
}

/// Deserialize a `.algo` file.
pub fn decode(bytes: &[u8]) -> Result<AlgoFile, String> {
    let mut r = Reader::new(bytes);
    if r.take(4)? != MAGIC {
        return Err("not an .algo file".into());
    }
    if r.u16()? != FORMAT_VERSION {
        return Err("incompatible .algo format version".into());
    }
    if r.u16()? != COMPILER_VERSION {
        return Err("incompatible compiler version".into());
    }
    let source_hash = r.u128()?;
    let nd = r.u32()? as usize;
    let mut deps = Vec::with_capacity(nd);
    for _ in 0..nd {
        let name = r.str()?;
        let h = r.u128()?;
        deps.push((name, h));
    }
    let ns = r.u32()? as usize;
    let strings = (0..ns).map(|_| r.str()).collect::<Result<_, _>>()?;
    let nr = r.u32()? as usize;
    let mut rewrite = RewriteSystem::new();
    for _ in 0..nr {
        let l = r_expr(&mut r)?;
        let rr = r_expr(&mut r)?;
        let nm = r.u32()? as usize;
        let metas = (0..nm).map(|_| r.sym()).collect::<Result<_, _>>()?;
        rewrite.push(l, rr, metas);
    }
    let ne = r.u32()? as usize;
    let exports = (0..ne).map(|_| r.sym()).collect::<Result<_, _>>()?;
    let no = r.u32()? as usize;
    let mut obligations = Vec::with_capacity(no);
    for _ in 0..no {
        let label = r.str()?;
        let root = r_step(&mut r)?;
        obligations.push(Obligation { label, root });
    }
    Ok(AlgoFile {
        source_hash,
        deps,
        strings,
        rewrite,
        exports,
        obligations,
    })
}
