// Tree-sitter grammar for the .alg algebraic specification language.
// Mirrors algae/parser.py: declarations (sort/param/op/eq/prop/lemma/rule/
// include/open/alias/let), propositions and sequents (with typed context
// variables), equational type expressions (incl. sort constructors and
// qualified names), structured goal/by/therefore/done proofs, and Pratt-parsed
// terms with quantifiers and lambda. Unicode symbols and their ASCII / keyword
// aliases are interchangeable. There are no built-in numeric sorts.

const PREC = {
  binder: 0, // λ / ∀ / ∃ / if / let — extend greedily to the right
  if: 0,
  let: 0,
  iff: 1,
  implies: 2,
  or: 3,
  and: 4,
  compare: 5, // = ≠
  pipe: 6, // ▷ pipe-last application sugar
  concat: 7, // ++ sequence concatenation
  unary: 8, // ¬
  method: 9, // . pipe-first application sugar
  postfix: 11, // call / prime
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
    // An empty `()` could open a binder list or be the unit value.
    [$.binders, $.unit],
    // A primed binder name `b'` vs a primed expression `b'` inside `(`.
    [$.decl_name, $._expression],
  ],

  rules: {
    source_file: $ => repeat($._declaration),

    _declaration: $ => choice(
      $.sort_declaration,
      $.param_declaration,
      $.op_declaration,
      $.eq_declaration,
      $.prop_declaration,
      $.lemma_declaration,
      $.rule_declaration,
      $.include_declaration,
      $.open_declaration,
      $.alias_declaration,
      $.let_declaration,
    ),

    // sort Nat : Sort;   sort List : Sort → Sort;
    sort_declaration: $ => seq(
      'sort',
      field('name', alias($.identifier, $.sort_identifier)),
      ':',
      field('kind', $._type),
      ';',
    ),

    // param T : Sort;   param M : Sort → Sort;
    param_declaration: $ => seq(
      'param',
      field('name', alias($.identifier, $.sort_identifier)),
      ':',
      field('kind', $._type),
      ';',
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

    // eq / prop / lemma name [binders] equation;  — the body is an equation
    // (an expression), not a sequent. Binder variables are schematic parameters.
    eq_declaration: $ => seq(
      'eq',
      field('name', $.decl_name),
      optional(field('parameters', $.binders)),
      field('body', $._expression),
      ';',
    ),

    prop_declaration: $ => seq(
      'prop',
      field('name', alias($.decl_name, $.prop_name)),
      optional(field('parameters', $.binders)),
      field('body', $._expression),
      ';',
    ),

    lemma_declaration: $ => seq(
      'lemma',
      field('name', alias($.decl_name, $.lemma_name)),
      optional(field('parameters', $.binders)),
      field('body', $._expression),
      ';',
      optional(field('proof', $.proof_block)),
    ),

    // A declaration name: an identifier with trailing primes allowed (assoc').
    decl_name: $ => seq($.identifier, repeat("'")),

    // rule name(params) (case name <prop> end;)* ───── conclusion end;
    rule_declaration: $ => seq(
      'rule',
      field('name', $.identifier),
      field('parameters', $.binders),
      repeat(field('premise', $.rule_premise)),
      $.rule_bar,
      field('conclusion', $._prop),
      'end',
      ';',
    ),

    rule_premise: $ => seq(
      'case',
      field('name', $.identifier),
      field('body', $._prop),
      'end',
      ';',
    ),

    // A line of one or more box-drawing dashes separates premises from goal.
    rule_bar: $ => token(/─+/),

    include_declaration: $ => seq(
      'include',
      field('module', $.module_path),
      optional(seq('with', '(', commaSep($.with_binding), ')')),
      optional(field('obligations', $.obligation_block)),
      ';',
    ),

    // props case name … qed; … <qed|wip> — discharges the module's instantiated
    // props; the block is a subproof, so it carries its own terminator.
    obligation_block: $ => seq('props', repeat($.case_block), field('terminator', $.terminator)),

    with_binding: $ => seq(
      field('name', $.identifier),
      ':=',
      field('value', $._type),
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

    // Proofs --------------------------------------------------------------

    proof_block: $ => seq(
      'proof',
      repeat($.proof_step),
      field('terminator', $.terminator),
      ';',
    ),

    // A subproof is closed by `qed`, or by `wip` when it is still work in
    // progress (uses the `wip` tactic); the marker is viral up through
    // enclosing subproofs.
    terminator: $ => choice('qed', 'wip'),

    // Simple step:  goal <state> by <rewrite|assumption> therefore <state|done> ;
    // Apply step:   goal <state> by apply <call> <cases> therefore <state|done> <terminator> ;
    // An apply is a subproof, so its cases follow the call and its terminator
    // follows the `therefore`.
    proof_step: $ => seq(
      'goal',
      field('goal', $._prop),
      'by',
      choice(
        seq(field('tactic', $.rewrite_tactic), 'therefore', field('result', choice($.done, $._prop)), ';'),
        seq(field('tactic', $.wip_tactic), 'therefore', field('result', choice($.done, $._prop)), ';'),
        seq(
          field('tactic', $.apply_tactic),
          repeat($.case_block),
          'therefore',
          field('result', choice($.done, $._prop)),
          field('terminator', $.terminator),
          ';',
        ),
      ),
    ),

    done: $ => 'done',

    // Discharge the goal provisionally (work in progress); closes its subproof
    // with `wip`.
    wip_tactic: $ => 'wip',

    // rewrite > theorem(args) with ( lhs := rhs )   (or < for right-to-left)
    rewrite_tactic: $ => seq(
      'rewrite',
      field('direction', choice('>', '<')),
      field('theorem', $.theorem),
      'with',
      '(',
      field('lhs', $._expression),
      ':=',
      field('rhs', $._expression),
      ')',
    ),

    theorem: $ => seq(
      field('name', alias($.decl_name, $.rule_name)),
      optional(field('arguments', $.arguments)),
    ),

    // apply rule(args)   — the cases and the subproof terminator straddle the
    // enclosing step's `therefore` (see proof_step).
    apply_tactic: $ => seq(
      'apply',
      field('rule', alias($.decl_name, $.rule_name)),
      field('arguments', $.arguments),
    ),

    // case name proof_step* (qed|wip);  (matched to a premise/obligation by name)
    case_block: $ => seq(
      'case',
      field('name', $.identifier),
      repeat($.proof_step),
      field('terminator', $.terminator),
      ';',
    ),

    // let name = expr;  (top level, no `in`) names a term shared by declarations
    let_declaration: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
      ';',
    ),

    // Propositions --------------------------------------------------------

    _prop: $ => choice($.sequent, $._expression),

    // context? ⊢ goal
    sequent: $ => seq(
      optional(sep1($._context_entry, ',')),
      choice(...TURNSTILE),
      field('goal', $._expression),
    ),

    _context_entry: $ => choice($.context_var, $.assumption),

    // x : T — a typed local variable in the sequent context
    context_var: $ => seq(
      field('name', $.identifier),
      ':',
      field('type', $._type),
    ),

    assumption: $ => choice(
      seq(field('name', $.identifier), ':=', field('value', $._expression)),
      field('value', $._expression),
    ),

    // Binder lists --------------------------------------------------------

    // ( a : A, b b' : B )  — shared by λ, ∀/∃, rule and eq/prop/lemma params
    binders: $ => seq('(', commaSep($.binder_entry), ')'),

    binder_entry: $ => seq(
      repeat1(field('name', alias($.decl_name, $.binder_name))),
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

    builtin_type: $ => choice('𝔹', 'Bool', 'Prop', 'Sort'),

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
        ['left', PREC.compare, choice('=', '≠', '!=', 'neq')],
        ['left', PREC.pipe, choice('▷', '|>')],
        ['left', PREC.concat, '++'],
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
      field('operator', choice('¬', 'not')),
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

    comment: $ => token(seq('#', /.*/)),
  },
});
