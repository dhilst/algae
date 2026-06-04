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

Unicode symbols and their ASCII aliases (keywords or symbolic spellings) are
interchangeable. The formatter emits Unicode by default and emits the aliases
below with `fmt --ascii`. Additional symbolic input spellings are accepted:
`->` for `→`, `==>` for `⟹`, `<==>` for `⟺`, `!=` for `≠`, `<=` for `≤`,
`>=` for `≥`, `&&` for `∧`, `||` for `∨`.

| Symbol | Alias | Meaning |
|--------|-------|---------|
| `×` | `*` (or `product`) | product |
| `→` | `arrow` | operation/function arrow |
| `ℕ` | `Nat` | natural numbers |
| `ℤ` | `Int` | integers |
| `ℝ` | `Real` | reals |
| `𝔹` | `Bool` | booleans |
| `¬` | `not` | negation |
| `∧` | `/\` (or `and`) | conjunction |
| `∨` | `\/` (or `or`) | disjunction |
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

`check` type-checks declarations and axioms (see Type Checking below); axioms
are not proved or model-checked. `var` declarations are read as implicitly
universally quantified over all axioms.

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
            | 'let' let_lhs '=' expr 'in' expr

let_lhs    ::= identifier
             | '(' binder ',' binder (',' binder)* ')'   # destructuring
binder     ::= identifier | '_'

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

### Destructuring Let

A `let ... in` pattern with two or more binders takes a product-typed value
apart, naming its components:

```
op pop : NEStack → Stack × Elem;

axiom let (rest, top) = pop(n) in size(rest) = size(n) - 1;
```

The semantics are equational: `let (a, b) = t in body` reads as
`t = (a, b) ⟹ body` with `a` and `b` fresh universally quantified variables.
`_` is a binder for components the body does not use; **each `_` is a distinct
fresh variable** (two `_` in one pattern do not equate the components), and
`_` is not valid anywhere else — not as a name, not in expressions.

Destructuring requires the value's type to be a product with exactly as many
components as the pattern has binders. A sum cannot be destructured — for
`pop : Stack → Stack × Elem | Error`, `let (rest, top) = pop(s)` is a type
error, because the `Error` branch has no components to bind. Destructuring is
only available in `let ... in` expressions, not in top-level `let`
declarations.

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
  Anything else (e.g. `x ▷ a + b`, which parses as `x ▷ (a + b)`) is rejected
  by the type checker.
- Numbers are integers, so `1.5` parses as `.` applied to `1` and `5`, not as
  a decimal literal (the type checker rejects it: `5` is not callable).

## Type Checking

`check` parses and then type-checks. The rules:

- Every identifier in an axiom must resolve: a local let binding, a declared
  `var`, an enum value, a top-level `let`, or an op.
- Ops may be **overloaded**: the same name with different domains is resolved
  by argument types. Unresolvable or ambiguous calls are errors.
- Number literals are `ℕ`. Numeric sorts are **strict**: `ℕ`, `ℤ`, and `ℝ`
  are distinct types and do not widen into one another. Arithmetic over mixed
  numeric operands synthesizes the widest operand type; unary `-` and `-` over
  `ℕ` operands yield `ℤ` (negation and subtraction leave the naturals).
- Sums **inject**: a term of type `T` is accepted where `T | Error` is
  expected. Sums do **not** implicitly narrow: a term of type `T | Error` is
  rejected where `T` is expected. To assert the happy path, the spec declares
  an explicit cast op — by convention `op cast : (T | Error) → T;` — and
  wraps the fallible term, e.g. `assign_role(cast(with_perm), u, r)`. This is
  what lets setup chains compose error-returning ops while keeping every
  narrowing visible in the spec. A sum-typed value can never be destructured.
- Comparisons and boolean operators yield `𝔹`; `< ≤ > ≥` and arithmetic
  require numerics; `++` requires matching `Seq` operands; equations `=`/`≠`
  require compatible operand types (either direction).
- An axiom body must type to `𝔹`.

Errors are reported as `<file>: type error at line <N>, <message>` with the
declaration's line. `check --syntax-only` skips type checking.

## CLI

```
algae.py check [--syntax-only] file.alg [file2.alg ...]
algae.py fmt [--ascii --inplace --no-valign] file.alg [file2.alg ...]
algae.py print file.alg [file2.alg ...]
```

- `check` prints `<file>.alg: ok`, a parse error
  (`<file>.alg: error at <line>, Expected <foo> found <bar>`), or type errors
  (`<file>.alg: type error at line <N>, <message>`). `--syntax-only` skips
  type checking.
- `fmt` prints formatted source, or rewrites files with `--inplace`.
- `fmt` converts aliases to Unicode by default; `--ascii` emits the canonical ASCII aliases.
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
