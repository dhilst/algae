/**
 * @file Tree-sitter grammar for the Algae v2 language (.alg)
 * @author Algae project
 * @license MIT
 *
 * Translated from lang-specs/spec.md sections 2 (Lexical Structure) and 3 (Grammar).
 * Both ASCII and Unicode operator forms are accepted (spec section 2.6).
 */

/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Proposition operator precedence, weakest to strongest (spec section 3.21).
const PREC = {
  bicond: 1,   // <=> / ⇔
  impl: 2,     // => / ⇒
  disj: 3,     // \/ / ∨
  conj: 4,     // /\ / ∧
  eq: 5,       // =
  neg: 6,      // ~ / ¬
  // term / type precedences
  sum: 1,      // | (sum type)
  product: 2,  // * / × (product type / kind)
  arrow: 1,    // -> (function type, right associative)
  infix: 3,    // term infix operators (+ - * / == < > <= >=)
  app: 10,     // application
};

module.exports = grammar({
  name: 'alg',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  word: $ => $.identifier,

  // Reported conflicts that the GLR parser resolves dynamically.
  conflicts: $ => [
    // term/prop atoms overlap on identifiers, qualified names, applications
    // and parenthesised groups inside a "( ... )"; the GLR parser
    // disambiguates from the surrounding context.
    [$._prop_atom, $._term_atom],
    [$._prop_atom, $._application_term],
  ],

  rules: {
    // 3.1 File
    source_file: $ => repeat($._top_decl),

    // 2.1 Comments
    comment: _ => token(seq('#', /.*/)),

    // 2.3 Identifiers
    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,

    // 2.4 Qualified identifiers: module.symbol
    qualified_identifier: $ => seq(
      field('module', $.identifier),
      '.',
      field('name', $.identifier),
    ),

    _ident_or_qualified: $ => choice(
      $.qualified_identifier,
      $.identifier,
    ),

    // 2.6 numeric symbol used as operator name, e.g. `0`
    numeric_symbol: _ => /[0-9]+/,

    // 2.6 symbolic operators usable as names (× is accepted wherever * is, to
    // match the lexer, since fmt may render the multiplication operator as ×)
    symbolic_operator: _ => choice('+', '-', '*', '×', '/', '==', '<=', '>=', '<', '>'),

    // 3.2 Top-Level Declarations
    _top_decl: $ => choice(
      $.import_decl,
      $.sort_decl,
      $.op_decl,
      $.axiom_decl,
      $.rule_decl,
      $.lemma_decl,
      $.theorem_decl,
      $.theory_decl,
      $.model_decl,
    ),

    // 3.3 Imports
    import_decl: $ => seq(
      'import',
      field('module', $.identifier),
      optional($.import_list),
      ';',
    ),

    import_list: $ => seq(
      '(',
      optional(seq(
        $.import_item,
        repeat(seq(',', $.import_item)),
        optional(','),
      )),
      ')',
    ),

    import_item: $ => seq(
      field('name', $.identifier),
      optional(seq('as', field('alias', $.identifier))),
    ),

    // 3.4 Sort Declarations
    sort_decl: $ => seq(
      'sort',
      $.sort_binding,
      repeat(seq(',', $.sort_binding)),
      ';',
    ),

    sort_binding: $ => seq(
      field('names', $.ident_list),
      ':',
      field('kind', $.kind_expr),
    ),

    // ident_list = ident { ident }
    ident_list: $ => prec.right(repeat1($.identifier)),

    // 3.5 Operator Declarations
    op_decl: $ => seq(
      'op',
      field('name', $.symbol),
      ':',
      field('signature', $.function_sig),
      ';',
    ),

    // symbol = qualified_or_unqualified_ident | numeric_symbol | symbolic_operator
    symbol: $ => choice(
      $._ident_or_qualified,
      $.numeric_symbol,
      $.symbolic_operator,
    ),

    // function_sig = [ type_expr ] "->" type_expr
    //
    // The spec writes the domain as a full type_expr, but a full type_expr is
    // itself a (right-associative) function type and would greedily swallow the
    // "->" that separates domain from codomain. The domain is therefore the
    // arrow-free prefix (a sum_type), which still covers products and sums,
    // e.g. `Nat * Nat -> Nat`, `A -> B | Err`, `Option(A) * (A -> Option(B)) -> ...`.
    function_sig: $ => seq(
      optional(field('domain', alias($._sum_type, $.type_expr))),
      choice('->', '→'),
      field('codomain', $.type_expr),
    ),

    // 3.6 Axioms
    axiom_decl: $ => seq(
      'axiom',
      field('name', $.identifier),
      field('params', $.formal_params),
      field('statement', $.sequent),
      ';',
    ),

    // 3.7 Rules
    rule_decl: $ => seq(
      'rule',
      field('name', $.identifier),
      field('params', $.formal_params),
      $.rule_body,
      'end',
      ';',
    ),

    rule_body: $ => seq(
      field('premises', $.premise_list),
      $.separator,
      field('conclusion', $.sequent),
    ),

    premise_list: $ => seq(
      $.sequent,
      repeat(seq(';', $.sequent)),
    ),

    // separator = 24+ dashes (ASCII) | 24+ box-drawing (Unicode)
    // Written as `{24}` + `*` rather than `{24,}`: tree-sitter's lexer compiles
    // an open-ended counted repetition as if it were exact, so `{24,}` would
    // stop after exactly 24 characters and treat any extra as an error.
    separator: _ => token(choice(
      /-{24}-*/,
      /─{24}─*/,
    )),

    // 3.8 Lemmas and Theorems
    lemma_decl: $ => seq(
      'lemma',
      field('name', $.identifier),
      optional(field('params', $.formal_params)),
      field('statement', $.sequent),
      ';',
      field('proof', $.proof_block),
    ),

    theorem_decl: $ => seq(
      'theorem',
      field('name', $.identifier),
      optional(field('params', $.formal_params)),
      field('statement', $.sequent),
      ';',
      field('proof', $.proof_block),
    ),

    // 3.9 Theories
    theory_decl: $ => seq(
      'theory',
      field('name', $.identifier),
      field('params', $.formal_params),
      'laws',
      repeat($._theory_item),
      'qed',
      ';',
    ),

    _theory_item: $ => choice(
      $.include_decl,
      $.law_decl,
    ),

    // 3.10 Theory Includes
    include_decl: $ => seq(
      'include',
      field('name', $.identifier),
      field('args', $.actual_args),
      ';',
    ),

    // 3.11 Laws (syntactically compatible with axioms)
    law_decl: $ => seq(
      'law',
      field('name', $.identifier),
      field('params', $.formal_params),
      field('statement', $.sequent),
      ';',
    ),

    // 3.12 Models
    model_decl: $ => seq(
      'model',
      field('name', $.identifier),
      'satisfies',
      field('theory', $.identifier),
      field('args', $.actual_args),
      'iff',
      'props',
      repeat($.model_law),
      choice('qed', 'wip'),
      ';',
    ),

    model_law: $ => seq(
      'law',
      field('name', $._ident_or_qualified),
      ';',
      field('proof', $.proof_block),
    ),

    // 3.13 Proof Blocks (closed by `qed`, or `wip` if in progress)
    proof_block: $ => seq(
      'proof',
      $._proof_body,
      choice('qed', 'wip'),
      ';',
    ),

    _proof_body: $ => choice(
      $.by_stmt_wip,
      $.by_stmt_zero,
      $.by_stmt_then,
      $.by_stmt_many,
    ),

    // by wip ;  — admit the goal (no proof required)
    by_stmt_wip: $ => seq('by', 'wip', ';'),

    // by_stmt_zero = "by" proof_ref ;  (0 subgoals: closes the goal)
    by_stmt_zero: $ => seq(
      'by',
      field('ref', $.proof_ref),
      ';',
    ),

    // by_stmt_then = "by" proof_ref "then" continuation proof_body
    // (1 subgoal: flat continuation in the same block)
    by_stmt_then: $ => seq(
      'by',
      field('ref', $.proof_ref),
      'then',
      field('goal', $.case_body),
      $._proof_body,
    ),

    // by_stmt_many = "by" proof_ref "cases" case_block { case_block } ("qed"|"wip") ;
    // (2+ subgoals: branching)
    by_stmt_many: $ => seq(
      'by',
      field('ref', $.proof_ref),
      'cases',
      repeat1($.case_block),
      choice('qed', 'wip'),
      ';',
    ),

    // 3.14 Proof References
    proof_ref: $ => seq(
      field('name', $._ident_or_qualified),
      optional(field('args', $.actual_args)),
    ),

    // Tactic / theory / model arguments may be terms (e.g. lambdas) or props
    // (e.g. an equality written with `_` holes), mirroring the unified
    // expression language.
    actual_args: $ => seq(
      '(',
      optional($.arg_list),
      ')',
    ),

    arg_list: $ => seq(
      $._arg,
      repeat(seq(',', $._arg)),
    ),

    _arg: $ => choice($.prop, $.term),

    term_list: $ => seq(
      $.term,
      repeat(seq(',', $.term)),
    ),

    // 3.15 Cases
    case_block: $ => seq(
      'case',
      $.case_body,
      field('proof', $.proof_block),
    ),

    // case_body = [ context ] sequent_goal
    case_body: $ => seq(
      optional($.context),
      $.sequent_goal,
    ),

    // sequent_goal = ("|-" | "⊢") prop ";"
    sequent_goal: $ => seq(
      $.turnstile,
      field('goal', $.prop),
      ';',
    ),

    turnstile: _ => choice('|-', '⊢'),

    // 3.16 Sequents
    sequent: $ => seq(
      optional(field('context', $.context)),
      $.turnstile,
      field('goal', $.prop),
    ),

    // 3.17 Contexts
    // context = context_entry { context_sep context_entry } context_sep?
    context: $ => prec.right(seq(
      $.context_entry,
      repeat(seq($._context_sep, $.context_entry)),
      optional($._context_sep),
    )),

    _context_sep: _ => choice(',', ';'),

    context_entry: $ => choice(
      $.term_binding,
      $.proof_binding,
    ),

    // term_binding = ident_list ":" type_expr
    term_binding: $ => seq(
      field('names', $.ident_list),
      ':',
      field('type', $.type_expr),
    ),

    // proof_binding = ident ":=" prop
    proof_binding: $ => seq(
      field('name', $.identifier),
      ':=',
      field('prop', $.prop),
    ),

    // 3.18 Formal Parameters
    formal_params: $ => seq(
      '(',
      optional(seq(
        $.formal_param,
        repeat(seq(',', $.formal_param)),
      )),
      ')',
    ),

    formal_param: $ => choice(
      $.term_binding,
      $.proof_binding,
    ),

    // 3.19 Kinds
    kind_expr: $ => $._kind_function,

    _kind_function: $ => prec.right(PREC.arrow, seq(
      $._kind_product,
      optional(seq(choice('->', '→'), $._kind_function)),
    )),

    _kind_product: $ => prec.left(PREC.product, seq(
      $._kind_atom,
      repeat(seq($._product_op, $._kind_atom)),
    )),

    _kind_atom: $ => choice(
      $.sort_kind,
      seq('(', $.kind_expr, ')'),
    ),

    sort_kind: _ => 'Sort',

    _product_op: _ => choice('*', '×'),

    // 3.20 Types
    type_expr: $ => $._function_type,

    _function_type: $ => prec.right(PREC.arrow, seq(
      $._sum_type,
      optional(seq(choice('->', '→'), $._function_type)),
    )),

    _sum_type: $ => prec.left(PREC.sum, seq(
      $._product_type,
      repeat(seq($.sum_op, $._product_type)),
    )),

    sum_op: _ => '|',

    _product_type: $ => prec.left(PREC.product, seq(
      $._type_atom,
      repeat(seq($._product_op, $._type_atom)),
    )),

    _type_atom: $ => choice(
      $.type_application,
      $.prop_type,
      $._ident_or_qualified,
      seq('(', $.type_expr, ')'),
    ),

    prop_type: _ => 'Prop',

    // type_application = qualified_or_unqualified_ident "(" [ type_expr_list ] ")"
    type_application: $ => prec(PREC.app, seq(
      field('constructor', $._ident_or_qualified),
      '(',
      optional(seq(
        $.type_expr,
        repeat(seq(',', $.type_expr)),
      )),
      ')',
    )),

    // 3.21 Propositions
    prop: $ => $._biconditional_prop,

    _biconditional_prop: $ => prec.left(PREC.bicond, seq(
      $._implication_prop,
      repeat(seq($.biconditional_op, $._implication_prop)),
    )),

    biconditional_op: _ => choice('<=>', '⇔'),

    _implication_prop: $ => prec.left(PREC.impl, seq(
      $._disjunction_prop,
      repeat(seq($.implication_op, $._disjunction_prop)),
    )),

    implication_op: _ => choice('=>', '⇒'),

    _disjunction_prop: $ => prec.left(PREC.disj, seq(
      $._conjunction_prop,
      repeat(seq($.disjunction_op, $._conjunction_prop)),
    )),

    disjunction_op: _ => choice('\\/', '∨'),

    _conjunction_prop: $ => prec.left(PREC.conj, seq(
      $._negation_prop,
      repeat(seq($.conjunction_op, $._negation_prop)),
    )),

    conjunction_op: _ => choice('/\\', '∧'),

    _negation_prop: $ => choice(
      $.negation,
      $.quantified_prop,
      $.equality_prop,
      $._prop_atom,
    ),

    negation: $ => prec.right(PREC.neg, seq(
      $.negation_op,
      $._negation_prop,
    )),

    negation_op: _ => choice('~', '¬'),

    // quantified_prop = ("forall" | "exists") binder "st" prop
    quantified_prop: $ => prec.right(seq(
      field('quantifier', choice('forall', '∀', 'exists', '∃')),
      field('binder', $.binder),
      'st',
      field('body', $.prop),
    )),

    // equality_prop = term "=" term
    equality_prop: $ => prec.left(PREC.eq, seq(
      field('left', $.term),
      '=',
      field('right', $.term),
    )),

    _prop_atom: $ => choice(
      $.false_prop,
      $.application,
      $._ident_or_qualified,
      seq('(', $.prop, ')'),
    ),

    false_prop: _ => 'False',

    // 3.22 Binders
    binder: $ => seq('(', $.term_binding, ')'),

    // 3.23 Terms
    term: $ => $._lambda_term,

    _lambda_term: $ => choice(
      $.lambda_term,
      $._infix_term,
    ),

    // lambda_term = ("lambda" | "λ") binder "st" term
    //
    // The spec restricts the body to a term, but throughout the standard
    // library lambda bodies are propositions (equalities such as
    // `bind(xs, f) = f(x)`, conjunctions, etc.). We therefore accept a prop or
    // a term; a prop already subsumes applications and identifiers, so this is
    // a strict superset of the spec's term body.
    lambda_term: $ => prec.right(seq(
      field('lambda', choice('lambda', 'λ')),
      field('binder', $.binder),
      'st',
      field('body', choice($.prop, $.term)),
    )),

    // infix_term = application_term { infix_op application_term }
    _infix_term: $ => prec.left(PREC.infix, seq(
      $._application_term,
      repeat(seq($.infix_op, $._application_term)),
    )),

    // infix_op = + | - | * | / | qualified_or_unqualified_ident  (× ≡ *)
    infix_op: $ => choice(
      '+', '-', '*', '×', '/',
      $._ident_or_qualified,
    ),

    // application_term = term_atom [ "(" [ term_list ] ")" ]
    _application_term: $ => choice(
      $.application,
      $._term_atom,
    ),

    application: $ => prec(PREC.app, seq(
      field('function', $._term_atom),
      '(',
      optional($.term_list),
      ')',
    )),

    _term_atom: $ => choice(
      $.hole,
      $._ident_or_qualified,
      $.numeric_symbol,
      $.symbolic_operator,
      seq('(', $.term, ')'),
    ),

    // `_` hole: sugar for a unary lambda.
    hole: _ => '_',
  },
});
