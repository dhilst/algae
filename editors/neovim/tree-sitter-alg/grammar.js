// Tree-sitter grammar for the .alg algebraic specification language.
// Mirrors algae/parser.py: declarations (sort/op/var/axiom), equational
// type expressions, and Pratt-parsed terms. Unicode symbols and their
// ASCII / keyword aliases are interchangeable.

const PREC = {
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
const PRODUCT = ['×', '*', 'product'];

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
    [$.axiom_name, $._expression],
  ],

  rules: {
    source_file: $ => repeat($._declaration),

    _declaration: $ => choice(
      $.sort_declaration,
      $.op_declaration,
      $.var_declaration,
      $.axiom_declaration,
      $.let_declaration,
    ),

    // sort A, B;  |  sort A = {x, y};
    sort_declaration: $ => seq(
      'sort',
      field('name', alias($.identifier, $.sort_identifier)),
      optional(choice(
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

    // op name : domain → type;  (empty domain for nullary operations)
    op_declaration: $ => seq(
      'op',
      field('name', $.identifier),
      ':',
      optional(field('domain', $.domain)),
      choice(...ARROW),
      field('codomain', $._type),
      ';',
    ),

    domain: $ => sep1($._type_primary, choice(...PRODUCT)),

    var_declaration: $ => seq(
      'var',
      field('name', $.identifier),
      repeat(seq(',', field('name', $.identifier))),
      ':',
      field('type', $._type),
      ';',
    ),

    // axiom name? body;  — the optional name is an identifier with trailing
    // primes; GLR disambiguates against the body's leading identifier, and
    // the dynamic precedences mirror algae/parser.py: a bare identifier (or
    // one followed by `(` or `'`) is the expression, not the name.
    axiom_declaration: $ => seq(
      'axiom',
      optional(field('name', $.axiom_name)),
      field('body', $._expression),
      ';',
    ),

    axiom_name: $ => prec.dynamic(-1, seq($.identifier, repeat("'"))),

    // let name = expr;  (top level, no `in`) names a term shared by axioms
    let_declaration: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
      ';',
    ),

    // Types -------------------------------------------------------------

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
      $.sequence_type,
      $.unit_type,
      $.parenthesized_type,
      alias($.identifier, $.type_identifier),
    ),

    builtin_type: $ => choice('ℕ', 'ℤ', 'ℝ', '𝔹', 'Nat', 'Int', 'Real', 'Bool'),

    // Only `Seq` is valid here today; accepting any constructor keeps the
    // parser robust and lets queries pick out `Seq` specifically.
    sequence_type: $ => seq(
      field('constructor', alias($.identifier, $.type_identifier)),
      '[',
      field('item', $._type),
      ']',
    ),

    unit_type: $ => seq('(', ')'),

    parenthesized_type: $ => seq('(', $._type, ')'),

    // Expressions ---------------------------------------------------------

    _expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
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
    ),

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

    string: $ => token(seq(
      '"',
      repeat(choice(/[^"\\]/, seq('\\', /./))),
      '"',
    )),

    comment: $ => token(seq('#', /.*/)),
  },
});
