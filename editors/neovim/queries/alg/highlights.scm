; Highlight queries for .alg algebraic specifications.
; Later patterns take priority in Neovim, so the generic identifier
; capture comes first and specific contexts override it below.

(identifier) @variable

(comment) @comment

; Literals
(boolean) @boolean

; Declaration keywords
[
  "sort"
  "param"
  "op"
  "eq"
  "prop"
  "lemma"
  "rule"
  "end"
  "proof"
  "by"
  "apply"
  "case"
  "goal"
  "rewrite"
  "therefore"
  "include"
  "open"
  "with"
  "alias"
  "props"
] @keyword

(done) @keyword
(terminator) @keyword
(wip_tactic) @keyword

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

; `Seq[...]` is the one built-in container constructor.
((type_application
  constructor: (type_identifier) @type.builtin)
  (#eq? @type.builtin "Seq"))

; Declared names
(op_declaration
  name: (identifier) @function)

(rule_declaration
  name: (identifier) @function)

(let_expression
  name: (identifier) @variable)

(tuple_pattern
  (identifier) @variable)

(wildcard) @variable.builtin

(let_declaration
  name: (identifier) @variable)

; Binder names (λ / ∀ / ∃ / rule and eq/prop/lemma parameters)
(binder_entry
  name: (binder_name) @variable.parameter)

; Typed context variables in a sequent (`n : Nat ⊢ …`)
(context_var
  name: (identifier) @variable.parameter)

; Named sequent assumptions (`h := …`)
(assumption
  name: (identifier) @label)

; Premise and proof-case names
(rule_premise
  name: (identifier) @label)
(case_block
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
  "*"
  "¬"
  "∧"
  "∨"
  "⟹"
  "⟺"
  "="
  "≠"
  "!="
  "<"
  ">"
  "&&"
  "||"
  "/\\"
  "\\/"
  "==>"
  "<==>"
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
] @keyword.operator

; Punctuation
[
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  ";"
  ","
  ":"
  "::"
] @punctuation.delimiter

; eq, prop, lemma, rule, and proof-rule names
(decl_name) @label
(prop_name) @label
(lemma_name) @label
(rule_name) @label
