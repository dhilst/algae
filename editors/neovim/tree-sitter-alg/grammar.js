// Tree-sitter grammar for the .alg algebraic specification language.
// Mirrors algae/parser.py: declarations (sort/op/var/axiom/lemma/rule/
// include/open/alias/let), propositions and sequents, equational type
// expressions (incl. parametric sorts and qualified names), and Pratt-parsed
// terms with quantifiers and lambda. Unicode symbols and their ASCII / keyword
// aliases are interchangeable.

const PREC = {
  binder: 0, // λ / ∀ / ∃ / if / let — extend greedily to the right
  if: 0,
  let: 0,
  iff: 1,
  implies: 2,
  or: 3,
  and: 4,
  compare: 5,
  pipe: 6,
  additive: 7,
  multiplicative: 8,
  unary: 9,
  method: 10,
  postfix: 11,
};

const TYPE_PREC = {
  sum: 1,
  function: 2,
  product: 3,
};

const ARROW = ['→', '->', 'arrow'];
const PARTIAL_ARROW = ['⇸', '-/->'];
const PRODUCT = ['×', '*', 'product'];
const TURNSTILE = ['⊢', '|-'];
const LAMBDA = ['λ', 'fun'];
const FORALL = ['∀', 'forall'];
const EXISTS = ['∃', 'exists'];

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}

function commaSep(rule) {
  return optional(sep1(rule, ','));
}

module.exports = grammar({
  name: 'alg',

  extras: $ => [/\s/, $.comment],

  word: $ => $.identifier,

  conflicts: $ => [
    // An assumption is an expression; a bare-expression proposition is too. The
    // turnstile decides which once it (doesn't) appear.
    [$._prop, $.assumption],
    // An empty `()` could open a binder list or be the unit value.
    [$.binders, $.unit],
    // A primed binder name `b'` vs a primed expression `b'` inside `(`.
    [$.axiom_name, $._expression],
  ],

  rules: {
    source_file: $ => repeat($._declaration),

    _declaration: $ => choice(
      $.sort_declaration,
      $.op_declaration,
      $.var_declaration,
      $.axiom_declaration,
      $.lemma_declaration,
      $.rule_declaration,
      $.include_declaration,
      $.open_declaration,
      $.alias_declaration,
      $.let_declaration,
    ),

    // sort A, B;  |  sort A = {x, y};  |  sort List[T];  (parametric)
    sort_declaration: $ => seq(
      'sort',
      field('name', alias($.identifier, $.sort_identifier)),
      optional(choice(
        seq('[', sep1(field('param', alias($.identifier, $.type_parameter)), ','), ']'),
        repeat1(seq(',', field('name', alias($.identifier, $.sort_identifier)))),
        seq('=', field('values', $.enum_values)),
      )),
      ';',
    ),

    enum_values: $ => seq(
      '{',
      commaSep(field('value', alias($.identifier, $.enum_value))),
      '}',
    ),

    // op name : domain → type;  (empty domain for nullary operations,
    // ⇸ for partial operations)
    op_declaration: $ => seq(
      'op',
      field('name', $.identifier),
      ':',
      optional(field('domain', $.domain)),
      choice(choice(...ARROW), $.partial_arrow),
      field('codomain', $._type),
      ';',
    ),

    partial_arrow: $ => choice(...PARTIAL_ARROW),

    // A top-level `|` folds the domain into a single sum-typed argument,
    // grouping as in codomains: A × B | C is (A × B) | C.
    domain: $ => sep1(sep1($._type_primary, choice(...PRODUCT)), '|'),

    var_declaration: $ => seq(
      'var',
      field('name', $.identifier),
      repeat(seq(',', field('name', $.identifier))),
      ':',
      field('type', $._type),
      ';',
    ),

    // axiom name [binders] prop;  |  axiom name = prop;
    // The name is required (trailing primes allowed). An axiom/lemma is a
    // quantified proposition; explicit binders ≡ forall over them.
    axiom_declaration: $ => seq(
      'axiom',
      field('name', $.axiom_name),
      choice(
        seq('=', field('body', $._prop)),
        seq(field('parameters', $.binders), field('body', $._prop)),
        field('body', $._prop),
      ),
      ';',
    ),

    axiom_name: $ => seq($.identifier, repeat("'")),

    // lemma name [binders] prop;  optionally followed by a proof block.
    lemma_declaration: $ => seq(
      'lemma',
      field('name', alias($.axiom_name, $.lemma_name)),
      choice(
        seq('=', field('body', $._prop)),
        seq(field('parameters', $.binders), field('body', $._prop)),
        field('body', $._prop),
      ),
      ';',
      optional(field('proof', $.proof_block)),
    ),

    // rule name(params) premise* ───── conclusion end
    rule_declaration: $ => seq(
      'rule',
      field('name', $.identifier),
      field('parameters', $.binders),
      repeat(field('premise', $._prop)),
      $.rule_bar,
      field('conclusion', $._prop),
      'end',
    ),

    // A line of one or more box-drawing dashes separates premises from goal.
    rule_bar: $ => token(/─+/),

    include_declaration: $ => seq(
      'include',
      field('module', $.module_path),
      optional(seq('with', '(', commaSep($.with_binding), ')')),
      ';',
    ),

    with_binding: $ => seq(
      field('name', $.identifier),
      ':=',
      field('type', $._type),
    ),

    open_declaration: $ => seq(
      'open',
      field('module', $.module_path),
      '(',
      sep1(field('name', $.identifier), ','),
      ')',
      ';',
    ),

    alias_declaration: $ => seq(
      'alias',
      field('name', $.identifier),
      '=',
      field('module', $.module_path),
      ';',
    ),

    module_path: $ => sep1($.identifier, '::'),

    proof_block: $ => seq(
      'proof',
      repeat($.proof_step),
      'qed',
      ';',
    ),

    proof_step: $ => choice(
      seq(field('term', $._expression), ';'),
      seq(
        '=',
        field('term', $._expression),
        'by',
        field('rule', alias($.axiom_name, $.rule_name)),
        ';',
      ),
      $.apply_step,
    ),

    // apply rule(args); case [binders] … qed;  (cases end at the next
    // non-`case` token; the apply has no closing `qed` of its own)
    apply_step: $ => seq(
      'apply',
      field('rule', alias($.axiom_name, $.rule_name)),
      field('arguments', $.arguments),
      ';',
      repeat1($.case_block),
    ),

    // case [ name := prop, … ⊢ goal ]  — the branch's explicit sequent
    case_block: $ => seq(
      'case',
      '[',
      field('subgoal', $.sequent),
      ']',
      repeat($.proof_step),
      'qed',
      ';',
    ),

    // let name = expr;  (top level, no `in`) names a term shared by axioms
    let_declaration: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
      ';',
    ),

    // Propositions --------------------------------------------------------

    _prop: $ => choice($.sequent, $._expression),

    // assumptions? ⊢ goal
    sequent: $ => seq(
      optional(sep1($.assumption, ',')),
      choice(...TURNSTILE),
      field('goal', $._expression),
    ),

    assumption: $ => choice(
      seq(field('name', $.identifier), ':=', field('value', $._expression)),
      field('value', $._expression),
    ),

    // Binder lists --------------------------------------------------------

    // ( a : A, b b' : B )  — shared by λ, ∀/∃, rule and axiom/lemma params
    binders: $ => seq('(', commaSep($.binder_entry), ')'),

    binder_entry: $ => seq(
      repeat1(field('name', alias($.axiom_name, $.binder_name))),
      ':',
      field('type', $._type),
    ),

    // Types ---------------------------------------------------------------

    _type: $ => choice(
      $.sum_type,
      $.function_type,
      $.product_type,
      $._type_primary,
    ),

    sum_type: $ => prec.left(TYPE_PREC.sum, seq(
      field('left', $._type),
      '|',
      field('right', $._type),
    )),

    function_type: $ => prec.right(TYPE_PREC.function, seq(
      field('left', $._type),
      choice(...ARROW),
      field('right', $._type),
    )),

    product_type: $ => prec.left(TYPE_PREC.product, seq(
      field('left', $._type),
      choice(...PRODUCT),
      field('right', $._type),
    )),

    _type_primary: $ => choice(
      $.builtin_type,
      $.type_application,
      $.qualified_type,
      $.unit_type,
      $.parenthesized_type,
      alias($.identifier, $.type_identifier),
    ),

    builtin_type: $ => choice('ℕ', 'ℤ', 'ℝ', '𝔹', 'Prop', 'Nat', 'Int', 'Real', 'Bool'),

    // List[T], Seq[T], list::List[Elem] — a constructor applied to type args.
    type_application: $ => seq(
      field('constructor', choice($.qualified_type, alias($.identifier, $.type_identifier))),
      '[',
      sep1(field('argument', $._type), ','),
      ']',
    ),

    qualified_type: $ => seq(
      alias($.identifier, $.type_identifier),
      repeat1(seq('::', alias($.identifier, $.type_identifier))),
    ),

    unit_type: $ => seq('(', ')'),

    parenthesized_type: $ => seq('(', $._type, ')'),

    // Expressions ---------------------------------------------------------

    _expression: $ => choice(
      $.identifier,
      $.qualified_identifier,
      $.number,
      $.boolean,
      $.builtin_type,
      $.unit,
      $.tuple,
      $.parenthesized_expression,
      $.call_expression,
      $.prime_expression,
      $.unary_expression,
      $.binary_expression,
      $.if_expression,
      $.let_expression,
      $.lambda_expression,
      $.quantifier_expression,
    ),

    qualified_identifier: $ => prec(PREC.postfix, seq(
      $.identifier,
      repeat1(seq('::', $.identifier)),
    )),

    binary_expression: $ => {
      const table = [
        ['right', PREC.iff, choice('⟺', '<==>', 'iff')],
        ['right', PREC.implies, choice('⟹', '==>', 'implies')],
        ['left', PREC.or, choice('∨', '||', '\\/', 'or')],
        ['left', PREC.and, choice('∧', '&&', '/\\', 'and')],
        ['left', PREC.compare, choice('=', '≠', '!=', 'neq', '<', '≤', '<=', 'leq', '>', '≥', '>=', 'geq')],
        ['left', PREC.pipe, choice('▷', '|>')],
        ['left', PREC.additive, choice('+', '-', '++')],
        ['left', PREC.multiplicative, choice('*', '/', '×', 'product')],
        ['left', PREC.method, '.'],
      ];
      return choice(...table.map(([assoc, precedence, operator]) =>
        (assoc === 'left' ? prec.left : prec.right)(precedence, seq(
          field('left', $._expression),
          field('operator', operator),
          field('right', $._expression),
        ))));
    },

    unary_expression: $ => prec(PREC.unary, seq(
      field('operator', choice('¬', 'not', '-')),
      field('operand', $._expression),
    )),

    call_expression: $ => prec(PREC.postfix, seq(
      field('function', $._expression),
      field('arguments', $.arguments),
    )),

    arguments: $ => seq('(', commaSep($._expression), ')'),

    prime_expression: $ => prec(PREC.postfix, seq(
      field('operand', $._expression),
      "'",
    )),

    if_expression: $ => prec.right(PREC.if, seq(
      'if',
      field('condition', $._expression),
      'then',
      field('consequence', $._expression),
      'else',
      field('alternative', $._expression),
    )),

    let_expression: $ => prec.right(PREC.let, seq(
      'let',
      field('name', choice($.identifier, $.tuple_pattern)),
      '=',
      field('value', $._expression),
      'in',
      field('body', $._expression),
    )),

    // λ (a : A, b : B) => body   (ASCII: fun (…) => body)
    lambda_expression: $ => prec.right(PREC.binder, seq(
      choice(...LAMBDA),
      field('parameters', $.binders),
      '=>',
      field('body', $._expression),
    )),

    // ∀ (a : A, b : B) st body   /   ∃ (…) st body
    quantifier_expression: $ => prec.right(PREC.binder, seq(
      field('quantifier', choice(...FORALL, ...EXISTS)),
      field('parameters', $.binders),
      'st',
      field('body', $._expression),
    )),

    // (a, _, b) — destructures a product; `_` binds an unused component
    tuple_pattern: $ => seq(
      '(',
      $._binder,
      ',',
      sep1($._binder, ','),
      ')',
    ),

    _binder: $ => choice($.identifier, $.wildcard),

    wildcard: $ => '_',

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    tuple: $ => seq('(', $._expression, ',', sep1($._expression, ','), ')'),

    unit: $ => seq('(', ')'),

    // Tokens ---------------------------------------------------------------

    boolean: $ => choice('true', 'false', '⊤', '⊥', 'truth', 'falsehood'),

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,

    number: $ => /[0-9]+/,

    comment: $ => token(seq('#', /.*/)),
  },
});
