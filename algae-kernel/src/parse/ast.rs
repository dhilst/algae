//! Surface syntax tree produced by the parser (spec §3).
//!
//! Terms and propositions share one [`Expr`] type: in the unified expression
//! language a proposition is just a `Prop`-valued expression (so a lambda body
//! may be an equality, and `P(x)` is an application used as a proposition).
//! Type annotations keep their own [`Type`] grammar.

use crate::diagnostics::Span;

#[derive(Clone, Debug)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

/// A qualified-or-unqualified identifier: `name` or `module.name`.
#[derive(Clone, Debug)]
pub struct QName {
    pub module: Option<Name>,
    pub name: Name,
    pub span: Span,
}

/// A declarable symbol: an identifier, a numeric symbol, or a symbolic
/// operator (e.g. `+`).
#[derive(Clone, Debug)]
pub enum Symbol {
    Name(QName),
    Number(String, Span),
    Op(SymOp, Span),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymOp {
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub decls: Vec<Decl>,
}

/// How a block terminates: `qed` (claimed complete) or `wip` (declared
/// incomplete). The declaration is checked against the block's actual taint
/// during elaboration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Close {
    Qed,
    Wip,
}

impl Close {
    pub fn is_wip(self) -> bool {
        matches!(self, Close::Wip)
    }
}

#[derive(Clone, Debug)]
pub enum Decl {
    Import(ImportDecl),
    Sort(SortDecl),
    Op(OpDecl),
    Axiom(AxiomDecl),
    Rule(RuleDecl),
    Lemma(LemmaDecl),
    Theorem(LemmaDecl),
    Theory(TheoryDecl),
    Model(ModelDecl),
}

#[derive(Clone, Debug)]
pub struct ImportDecl {
    pub module: Name,
    pub items: Option<Vec<ImportItem>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImportItem {
    pub name: Name,
    pub alias: Option<Name>,
}

#[derive(Clone, Debug)]
pub struct SortDecl {
    pub bindings: Vec<SortBinding>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct SortBinding {
    pub names: Vec<Name>,
    pub kind: Kind,
}

#[derive(Clone, Debug)]
pub struct OpDecl {
    pub symbol: Symbol,
    pub sig: FunctionSig,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionSig {
    pub domain: Option<Type>,
    pub codomain: Type,
}

#[derive(Clone, Debug)]
pub struct AxiomDecl {
    pub name: Name,
    pub params: Vec<FormalParam>,
    pub sequent: Sequent,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RuleDecl {
    pub name: Name,
    pub params: Vec<FormalParam>,
    pub premises: Vec<Sequent>,
    pub conclusion: Sequent,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LemmaDecl {
    pub name: Name,
    pub params: Vec<FormalParam>,
    pub sequent: Sequent,
    pub proof: ProofBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TheoryDecl {
    pub name: Name,
    pub params: Vec<FormalParam>,
    pub items: Vec<TheoryItem>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TheoryItem {
    Include(IncludeDecl),
    Law(LawDecl),
}

#[derive(Clone, Debug)]
pub struct IncludeDecl {
    pub name: Name,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LawDecl {
    pub name: Name,
    pub params: Vec<FormalParam>,
    pub sequent: Sequent,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModelDecl {
    pub name: Name,
    pub theory: Name,
    pub args: Vec<Expr>,
    pub laws: Vec<ModelLaw>,
    /// How the `props` block is terminated.
    pub close: Close,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ModelLaw {
    pub law: QName,
    pub proof: ProofBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum FormalParam {
    Term(TermBinding),
    Proof(ProofBinding),
}

#[derive(Clone, Debug)]
pub struct TermBinding {
    pub names: Vec<Name>,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct ProofBinding {
    pub name: Name,
    pub prop: Expr,
}

#[derive(Clone, Debug)]
pub struct Sequent {
    pub context: Vec<ContextEntry>,
    pub prop: Expr,
    pub span: Span,
}

pub type ContextEntry = FormalParam;

#[derive(Clone, Debug)]
pub struct ProofBlock {
    /// A proof block contains exactly one `by` statement.
    pub stmt: ProofStmt,
    /// How this proof block is terminated.
    pub close: Close,
    /// Span of the `qed`/`wip` terminator keyword (for a precise fix that swaps
    /// just the keyword). Covers the whole block-terminating keyword token.
    pub close_span: Span,
    pub span: Span,
}

/// Which surface form linked this `by` step to its continuation. Affects
/// diagnostics only — `Then` and `Cases` both lower to `cases`, and the checker
/// treats them identically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cont {
    /// `by ref;` — closes the goal (or `by wip`); no continuation.
    Zero,
    /// `by ref then <seq> …` — single-goal continuation (one synthetic case).
    Then,
    /// `by ref cases case … case …` — multi-goal branching.
    Cases,
}

#[derive(Clone, Debug)]
pub struct ProofStmt {
    /// `None` for an admit (`by wip`).
    pub reference: Option<ProofRef>,
    /// True for `by wip` (admits the goal).
    pub admit: bool,
    pub cases: Vec<CaseBlock>,
    /// For a multi-case (`cases … qed/wip`) statement: how it is terminated.
    pub cases_close: Close,
    /// Span of the `qed`/`wip` keyword that closes a `cases` block (for a fix
    /// that swaps just the keyword). Meaningful only when `continuation` is
    /// `Cases`; a dummy span otherwise.
    pub cases_close_span: Span,
    /// Which surface form produced `cases` (for diagnostics).
    pub continuation: Cont,
    /// For `by wip(?name)`: the hole's name (without the `?`). Triggers a hole
    /// report at the current goal instead of a silent admit.
    pub hole: Option<String>,
    /// True for a tactic-inspect step: `by ref(…)?`, a `?name` argument, or a
    /// terminal `then ?name`. Reports argument holes / the next goal, then admits.
    pub inspect: bool,
    /// Name for the resulting-subgoal hole, from `then ?name` on an inspect step.
    pub subgoal_name: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ProofRef {
    pub name: QName,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct CaseBlock {
    pub context: Vec<ContextEntry>,
    pub goal: Expr,
    pub proof: ProofBlock,
    pub span: Span,
}

// ---- Kinds ----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Kind {
    pub node: KindNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum KindNode {
    Sort,
    Product(Vec<Kind>),
    Arrow(Box<Kind>, Box<Kind>),
}

// ---- Types ----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TypeNode {
    Name(QName),
    App(QName, Vec<Type>),
    Sort,
    Prop,
    Product(Vec<Type>),
    Sum(Vec<Type>),
    Arrow(Box<Type>, Box<Type>),
}

// ---- Binders --------------------------------------------------------------

/// A binder `( ident_list : type )`.
#[derive(Clone, Debug)]
pub struct Binder {
    pub names: Vec<Name>,
    pub ty: Type,
    pub span: Span,
}

// ---- Expressions (terms and propositions unified) -------------------------

#[derive(Clone, Debug)]
pub struct Expr {
    pub node: ExprNode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Var(QName),
    Num(String),
    Op(SymOp),
    App(Box<Expr>, Vec<Expr>),
    Infix(Box<Expr>, InfixOp, Box<Expr>),
    Lambda(Binder, Box<Expr>),
    Forall(Binder, Box<Expr>),
    Exists(Binder, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Implies(Box<Expr>, Box<Expr>),
    Iff(Box<Expr>, Box<Expr>),
    False,
    /// `_` hole: sugar for a unary lambda (expanded during elaboration).
    Hole,
    /// `?name` hole: a named argument hole, valid only in a tactic-inspect step
    /// (`by ref(… ?name …)`); reported rather than lowered to a term.
    NamedHole(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfixOp {
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Lt,
    Gt,
    Le,
    Ge,
}
