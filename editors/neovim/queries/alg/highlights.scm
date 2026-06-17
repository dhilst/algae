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
  "lemma"
  "rule"
  "end"
  "proof"
  "qed"
  "by"
  "apply"
  "case"
  "include"
  "open"
  "with"
  "alias"
] @keyword

[
  "let"
  "in"
] @keyword

; Quantifier separator
"st" @keyword

[
  "if"
  "then"
  "else"
] @keyword.conditional

; Quantifiers and lambda (Unicode and word aliases)
[
  "∀"
  "∃"
  "forall"
  "exists"
  "λ"
  "fun"
] @keyword.operator

; Types
(sort_identifier) @type
(type_identifier) @type
(builtin_type) @type.builtin
(type_parameter) @type.parameter

; `Seq[...]` is the one built-in container constructor.
((type_application
  constructor: (type_identifier) @type.builtin)
  (#eq? @type.builtin "Seq"))

(enum_value) @constant

; Declared names
(op_declaration
  name: (identifier) @function)

(rule_declaration
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

; Binder names (λ / ∀ / ∃ / rule and axiom/lemma parameters)
(binder_entry
  name: (binder_name) @variable.parameter)

; Named sequent assumptions (rule premises and the hypotheses a case binds in
; its explicit subgoal `case [ h := … ⊢ … ]`)
(assumption
  name: (identifier) @label)

(call_expression
  function: (identifier) @function.call)

; Module paths and qualified names. In `foo::bar::name`, the leading segments
; are namespaces and the final segment is the name (overridden below).
(module_path (identifier) @module)
(alias_declaration name: (identifier) @module)
(qualified_identifier (identifier) @module)
(qualified_identifier (identifier) @variable .)
(call_expression
  function: (qualified_identifier (identifier) @function.call .))

; Symbolic operators
[
  "→"
  "->"
  "⇸"
  "-/->"
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
  "⊢"
  "|-"
  "=>"
  ":="
] @operator

(rule_bar) @operator

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
  "::"
] @punctuation.delimiter

; Axiom, lemma, and proof rule names
(axiom_name) @label
(lemma_name) @label
(rule_name) @label
