# `.alg` Language Reference

## File Format

- Extension: `.alg`
- Encoding: UTF-8
- Comments: `#` to end of line
- Whitespace is insignificant.
- A file contains top-level algebraic declarations; there is no `spec` wrapper.
- Declarations end with `;`.

## Keywords

```
sort  op  var  axiom  true  false  if  then  else  let  in
```

The previous state-machine syntax is not part of this grammar. Its keywords
(`spec`, `state`, `init`, `pre`, `post`, ...) are ordinary identifiers now, but
old-syntax files still fail to parse since declarations must start with `sort`,
`op`, `var`, or `axiom`.

## Symbols And ASCII Keyword Aliases

Unicode symbols and their lowercase ASCII keyword aliases are interchangeable.
The formatter emits Unicode by default and emits aliases with `fmt --ascii`.

| Symbol | Keyword | Meaning |
|--------|---------|---------|
| `×` | `product` | product |
| `→` | `arrow` | operation/function arrow |
| `ℕ` | `nat` | natural numbers |
| `ℤ` | `int` | integers |
| `ℝ` | `real` | reals |
| `𝔹` | `bool` | booleans |
| `¬` | `not` | negation |
| `∧` | `and` | conjunction |
| `∨` | `or` | disjunction |
| `⟹` | `implies` | implication |
| `⟺` | `iff` | biconditional |
| `≠` | `neq` | not equal |
| `≤` | `leq` | less or equal |
| `≥` | `geq` | greater or equal |
| `⊤` | `truth` | logical top |
| `⊥` | `falsehood` | logical bottom |

Set-theory notation (`∈`, `⊆`, `∪`, `∩`, `∅`, `℘`, `∀`/`∃` quantifiers, set and
mapping literals) is not part of the grammar. Specifications are equational:
behavior is captured by axioms over constructor terms, with `var` declarations
read as implicitly universally quantified.

`empty` and `top` are ordinary identifiers so they can name operations.

## Grammar

```
file       ::= decl*
decl       ::= sort_decl | op_decl | var_decl | axiom_decl

sort_decl  ::= 'sort' identifier (',' identifier)* ';'
             | 'sort' identifier '=' '{' identifier (',' identifier)* '}' ';'

op_decl    ::= 'op' identifier ':' domain '→' type_expr ';'
domain     ::=                    # empty domain for nullary operations
             | type_product

var_decl   ::= 'var' identifier ':' type_expr ';'
axiom_decl ::= 'axiom' expr ';'
```

## Type Expressions

```
type_expr    ::= type_sum
type_sum     ::= type_arrow ('|' type_arrow)*       # algebraic sum/union type
type_arrow   ::= type_product ('→' type_arrow)?
type_product ::= type_primary ('×' type_primary)*
type_primary ::= identifier | 'ℕ' | 'ℤ' | 'ℝ' | '𝔹'
               | 'Seq' '[' type_expr ']'
               | '(' type_expr ')'
               | '()'
```

## Terms And Axioms

The parser checks syntax only. It does not type-check variables, arity, or axiom
validity. `var` declarations are conventionally read as implicitly universally
quantified over all axioms.

```
expr      ::= identifier
            | literal
            | expr '(' args ')'
            | '(' expr ')'
            | expr comparison expr
            | expr bool_op expr
            | 'if' expr 'then' expr 'else' expr
            | 'let' identifier '=' expr 'in' expr

comparison ::= '=' | '≠' | '<' | '≤' | '>' | '≥'
bool_op    ::= '∧' | '∨' | '⟹' | '⟺'
```

`let` names an intermediate term so deeply nested axioms stay readable. Lets
nest, so a chain of bindings reads top to bottom:

```
axiom let with_user = add_user(rbac, u) in
      let with_role = add_role(with_user, r) in
      authorized(with_role, u, p) = true;
```

## CLI

```
algae.py check file.alg [file2.alg ...]
algae.py fmt [--ascii --inplace] file.alg [file2.alg ...]
algae.py print file.alg [file2.alg ...]
```

- `check` prints `<file>.alg: ok` or `<file>.alg: error at <line>, Expected <foo> found <bar>`.
- `fmt` prints formatted source, or rewrites files with `--inplace`.
- `fmt` converts aliases to Unicode by default; `--ascii` emits lowercase keyword aliases.
- `print` emits the parsed AST as JSON.

## Example

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : -> Stack;
op push  : Stack × Elem -> Stack;
op pop   : Stack -> Stack | Error;
op top   : Stack -> Elem | Error;

var s : Stack;
var e : Elem;

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
```
