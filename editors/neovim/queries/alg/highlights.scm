; Highlight queries for .alg algebraic specifications.
; Later patterns take priority in Neovim, so the generic identifier
; capture comes first and specific contexts override it below.

(identifier) @variable

(comment) @comment

; Literals
(number) @number
(string) @string
(boolean) @boolean

; Declaration keywords
[
  "sort"
  "op"
  "var"
  "axiom"
] @keyword

[
  "let"
  "in"
] @keyword

[
  "if"
  "then"
  "else"
] @keyword.conditional

; Types
(sort_identifier) @type
(type_identifier) @type
(builtin_type) @type.builtin

((sequence_type
  constructor: (type_identifier) @type.builtin)
  (#eq? @type.builtin "Seq"))

(enum_value) @constant

; Declared names
(op_declaration
  name: (identifier) @function)

(var_declaration
  name: (identifier) @variable.parameter)

(let_expression
  name: (identifier) @variable)

(tuple_pattern
  (identifier) @variable)

(wildcard) @variable.builtin

(let_declaration
  name: (identifier) @variable)

(call_expression
  function: (identifier) @function.call)

; Symbolic operators
[
  "→"
  "->"
  "×"
  "¬"
  "∧"
  "∨"
  "⟹"
  "⟺"
  "="
  "≠"
  "!="
  "<"
  "≤"
  "<="
  ">"
  "≥"
  ">="
  "&&"
  "||"
  "/\\"
  "\\/"
  "==>"
  "<==>"
  "+"
  "-"
  "*"
  "/"
  "++"
  "|"
  "'"
  "▷"
  "|>"
  "."
] @operator

; Word aliases for operators
[
  "arrow"
  "product"
  "not"
  "and"
  "or"
  "implies"
  "iff"
  "neq"
  "leq"
  "geq"
] @keyword.operator

; Punctuation
[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  ";"
  ","
  ":"
] @punctuation.delimiter

; Axiom names
(axiom_name) @label
