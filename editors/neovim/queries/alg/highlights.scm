; Tree-sitter highlight queries for Algae v2 (.alg)
; Capture groups follow the standard nvim-treesitter / Helix conventions.
;
; NOTE on precedence: Neovim's tree-sitter highlighter lets a LATER matching
; pattern override an earlier one for the same node. The generic
; `(identifier) @variable` fallback is therefore placed FIRST so that the more
; specific rules below take priority over it.

; ---------------------------------------------------------------------------
; Fallback: bare identifiers are variables (overridden by specifics below).
; ---------------------------------------------------------------------------
(identifier) @variable

; ---------------------------------------------------------------------------
; Comments
; ---------------------------------------------------------------------------
(comment) @comment @spell

; ---------------------------------------------------------------------------
; Reserved keywords (spec section 2.5)
; ---------------------------------------------------------------------------
[
  "import"
  "sort"
  "op"
  "axiom"
  "rule"
  "lemma"
  "theorem"
  "theory"
  "law"
  "model"
  "include"
  "end"
] @keyword

[
  "proof"
  "qed"
  "by"
  "case"
] @keyword

; Modifier-ish / relational keywords.
[
  "satisfies"
  "iff"
  "as"
] @keyword.modifier

; Quantifiers and binder keywords are operator-flavoured keywords.
[
  "forall"
  "exists"
  "st"
  "lambda"
  "λ"
] @keyword.operator

; ---------------------------------------------------------------------------
; Built-in sorts / type constants
; ---------------------------------------------------------------------------
(sort_kind) @type.builtin       ; Sort
(prop_type) @type.builtin       ; Prop
(false_prop) @constant.builtin  ; False

; ---------------------------------------------------------------------------
; Declaration names (the thing being defined)
; ---------------------------------------------------------------------------
(sort_decl (sort_binding names: (ident_list (identifier) @type)))

(op_decl name: (symbol (identifier) @function))
(op_decl name: (symbol (numeric_symbol) @function))
(op_decl name: (symbol (symbolic_operator) @function))

(axiom_decl name: (identifier) @function)
(rule_decl name: (identifier) @function)
(lemma_decl name: (identifier) @function)
(theorem_decl name: (identifier) @function)
(law_decl name: (identifier) @function)
(theory_decl name: (identifier) @type)
(model_decl name: (identifier) @type)
(model_decl theory: (identifier) @type)
(include_decl name: (identifier) @type)

(import_decl module: (identifier) @module)
(import_item name: (identifier) @variable)
(import_item alias: (identifier) @variable)

; ---------------------------------------------------------------------------
; Proof references and law selection inside models
; ---------------------------------------------------------------------------
(proof_ref name: (identifier) @function.call)
(model_law name: (identifier) @function.call)

; ---------------------------------------------------------------------------
; Qualified identifiers: module.name
; ---------------------------------------------------------------------------
(qualified_identifier
  module: (identifier) @module
  "." @punctuation.delimiter
  name: (identifier) @variable)

; ---------------------------------------------------------------------------
; Bindings
; ---------------------------------------------------------------------------
(term_binding names: (ident_list (identifier) @variable.parameter))
(proof_binding name: (identifier) @variable.parameter)
(binder (term_binding names: (ident_list (identifier) @variable.parameter)))

; ---------------------------------------------------------------------------
; Function / type applications
; ---------------------------------------------------------------------------
(application function: (identifier) @function.call)
(type_application constructor: (identifier) @type)

; ---------------------------------------------------------------------------
; Numbers (numeric symbols used as terms / operator names)
; ---------------------------------------------------------------------------
(numeric_symbol) @number

; ---------------------------------------------------------------------------
; Logical / sequent operators
; ---------------------------------------------------------------------------
(turnstile) @keyword.operator      ; |-  /  ⊢
(separator) @punctuation.special   ; ------ / ────── inference line

[
  (negation_op)        ; ~  / ¬
  (conjunction_op)     ; /\ / ∧
  (disjunction_op)     ; \/ / ∨
  (implication_op)     ; => / ⇒
  (biconditional_op)   ; <=> / ⇔
] @keyword.operator

; ---------------------------------------------------------------------------
; Term / type operators
; ---------------------------------------------------------------------------
(infix_op) @operator
(symbolic_operator) @operator
(sum_op) @operator           ; |  (sum type)

[
  "->"
  "→"
  "="
  "*"
  "×"
] @operator

":=" @operator

; ---------------------------------------------------------------------------
; Punctuation
; ---------------------------------------------------------------------------
[
  ":"
  ","
  ";"
] @punctuation.delimiter

[
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket
