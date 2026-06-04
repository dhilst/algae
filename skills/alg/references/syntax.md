# `.alg` Language Reference

## File Format

- Extension: `.alg`
- Encoding: UTF-8
- Comments: `#` to end of line, as in bash. `fmt` preserves them: standalone
  comment lines stay above the next declaration, and a trailing comment stays
  on its declaration's line.
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

## Symbols And ASCII Aliases

Unicode symbols and their ASCII aliases (lowercase keywords or symbolic
spellings) are interchangeable. The formatter emits Unicode by default and
emits aliases with `fmt --ascii`.

| Symbol | Alias | Meaning |
|--------|-------|---------|
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
| `▷` | `\|>` | pipe-last application sugar |

Set-theory notation (`∈`, `⊆`, `∪`, `∩`, `∅`, `℘`, `∀`/`∃` quantifiers, set and
mapping literals) is not part of the grammar. Specifications are equational:
behavior is captured by axioms over constructor terms, with `var` declarations
read as implicitly universally quantified.

`empty` and `top` are ordinary identifiers so they can name operations.

## Grammar

```
file       ::= decl*
decl       ::= sort_decl | op_decl | var_decl | axiom_decl | let_decl

sort_decl  ::= 'sort' identifier (',' identifier)* ';'
             | 'sort' identifier '=' '{' identifier (',' identifier)* '}' ';'

op_decl    ::= 'op' identifier ':' domain '→' type_expr ';'
domain     ::=                    # empty domain for nullary operations
             | type_product

var_decl   ::= 'var' identifier ':' type_expr ';'
axiom_decl ::= 'axiom' expr ';'
let_decl   ::= 'let' identifier '=' expr ';'
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
            | expr '.' expr
            | expr '▷' expr
            | 'if' expr 'then' expr 'else' expr
            | 'let' identifier '=' expr 'in' expr

comparison ::= '=' | '≠' | '<' | '≤' | '>' | '≥'
bool_op    ::= '∧' | '∨' | '⟹' | '⟺'
```

`let` names an intermediate term so deeply nested axioms stay readable. Lets
nest, so a chain of bindings reads top to bottom; the formatter breaks the
line after each `in` and aligns the bindings:

```
axiom let with_user = add_user(rbac, u) in
      let with_role = add_role(with_user, r) in
      authorized(with_role, u, p) = true;
```

A `let` may also appear at top level (no `in`), naming a term once for every
axiom that follows. This avoids repeating a common setup chain:

```
let with_user = add_user(empty_rbac(), u);
let with_role = add_role(with_user, r);

axiom authorized(with_role, u, p) = false;
axiom let revoked = remove_user(with_role, u) in authorized(revoked, u, p) = unknown_user;
```

Top-level lets are abbreviations: the parser records them but does not check
that axioms reference them, and variables inside the named term are still read
as universally quantified per axiom.

## Application Sugar

`.` and `▷` are infix application sugar. Both insert their left operand as an
argument of the call (or bare operation name) on their right; they differ in
which position it lands:

| Sugar | Reads as | Style |
|-------|----------|-------|
| `x.f(a, b)` | `f(x, a, b)` | pipe-first (method/UFCS) |
| `x ▷ f(a, b)` | `f(a, b, x)` | pipe-last (OCaml/F# pipe) |
| `x.f` and `x ▷ f` | `f(x)` | both agree on bare names |

Both are left-associative, so chains thread the running value step by step:

```
axiom s.push(e).pop = (s, e);                      # pop(push(s, e))
axiom s |> push(e) |> pop = (e, s);                # with data-last signatures
let store = empty_rbac().add_user(u).add_role(r);  # builder-style setup
```

`.` binds tightly (above `*`, below calls); `▷` binds loosely (above the
comparisons, below `+`), so `a + 1 ▷ f` reads `f(a + 1)` and
`s ▷ f = e` reads `(s ▷ f) = e`.

### Data-First Style (`.`)

Put the structure first in every domain, and chain with `.`. This mirrors
object-oriented targets, where the structure is the receiver and each `op`
becomes a method:

```
op push : Stack × Elem → Stack;          # stack.push(elem) in the target
op pop  : Stack → Stack × Elem | Error;

axiom s.push(e).pop = (s, e);            # pop(push(s, e))
axiom empty().pop = empty_error;
```

Prefer this style when the spec targets object-oriented code (Python classes,
Java, methods on a struct): `s.push(e).pop` reads as the method chain the
implementation will actually have.

### Data-Last Style (`▷`)

Put the structure last in every domain, and chain with `▷`. This mirrors
functional targets, where structure-last signatures curry into pipelines
(`push : elem -> stack -> stack` in OCaml, so `push e` partially applies and
`s |> push e` pipes):

```
op push : Elem × Stack → Stack;          # push elem stack in the target
op pop  : Stack → Elem × Stack | Error;

axiom s ▷ push(e) ▷ pop = (e, s);        # pop(push(e, s))
axiom empty() ▷ pop = empty_error;
```

Prefer this style when the spec targets functional code (OCaml, Haskell, F#,
Elm), where data-last is the standard library convention.

Match the spec's style to its implementation target and keep one style per
spec: the argument order of every `op`, the pair order of returned products,
and the choice of `.` versus `▷` should all agree.

Caveats:

- The reading is defined when the right operand is a call or a bare name.
  Anything else (e.g. `x ▷ a + b`, which parses as `x ▷ (a + b)`) still
  parses — the tool is syntax-only — but has no defined meaning.
- Numbers are integers, so `1.5` parses as `.` applied to `1` and `5`, not as
  a decimal literal.

## CLI

```
algae.py check file.alg [file2.alg ...]
algae.py fmt [--ascii --inplace --no-valign] file.alg [file2.alg ...]
algae.py print file.alg [file2.alg ...]
```

- `check` prints `<file>.alg: ok` or `<file>.alg: error at <line>, Expected <foo> found <bar>`.
- `fmt` prints formatted source, or rewrites files with `--inplace`.
- `fmt` converts aliases to Unicode by default; `--ascii` emits lowercase keyword aliases.
- `fmt` aligns separators vertically within each run of same-kind
  declarations (`:` for op/var blocks, `=` for let blocks and single-line `=`
  axioms); a standalone comment starts a new run. `--no-valign` disables the
  padding.
- `print` emits the parsed AST as JSON.

## Example

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : → Stack;
op push  : Stack × Elem → Stack;
op pop   : Stack → Stack × Elem | Error;
op top   : Stack → Elem | Error;

var s : Stack;
var e : Elem;

axiom s.push(e).pop = (s, e);
axiom s.push(e).top = e;
axiom empty().pop   = empty_error;
axiom empty().top   = empty_error;
```
