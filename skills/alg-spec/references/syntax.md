# `.alg` Language Reference

## File Format

- Extension: `.alg`
- Encoding: UTF-8
- Comments: `#` to end of line
- Indentation: 2 spaces (significant for block structure)

## Keywords

```
spec  extends  import  type  state  init  inv  op  pre  post  ret  prop  fn
```

## Symbols

| Symbol | Meaning | ASCII fallback |
|--------|---------|----------------|
| `∈`  | element of | `in` |
| `∉`  | not element of | `not in` |
| `⊆`  | subset or equal | `<=` |
| `⊂`  | proper subset | `<` |
| `⊇`  | superset or equal | `>=` |
| `∪`  | union | `\|` |
| `∩`  | intersection | `&` |
| `\`  | set difference | `\` |
| `×`  | cartesian product | `*` |
| `→`  | function type / returns | `->` |
| `↦`  | maps to (in bindings) | `\|->` |
| `∅`  | empty set/map | `{}` |
| `ℕ`  | natural numbers (0,1,2,...) | `Nat` |
| `ℤ`  | integers | `Int` |
| `ℝ`  | reals | `Real` |
| `𝔹`  | booleans | `Bool` |
| `∀`  | for all | `forall` |
| `∃`  | exists | `exists` |
| `¬`  | negation | `!` |
| `∧`  | conjunction | `&&` |
| `∨`  | disjunction | `\|\|` |
| `⟹` | implication | `==>` |
| `⟺` | biconditional | `<==>` |
| `℘`  | power set | `P` |
| `'`  | post-state suffix | `'` |
| `·`  | separator (in quantifiers) | `.` |
| `≠`  | not equal | `!=` |
| `≤`  | less or equal | `<=` |
| `≥`  | greater or equal | `>=` |

Unicode and ASCII forms are interchangeable. Use whichever your editor supports.

## Grammar

### Top-level declarations

```
file       ::= (import | spec)*
import     ::= 'import' path
path       ::= identifier ('/' identifier)*
spec       ::= 'spec' Name ('[' params ']')? ('extends' Name ('[' args ']')?)? body
params     ::= identifier (',' identifier)*
args       ::= type_expr (',' type_expr)*
```

### Spec body

```
body       ::= (type_def | state_block | init_block | inv | op | prop | fn)*
type_def   ::= 'type' Name '=' type_expr
state_block::= 'state' (identifier ':' type_expr)+
init_block ::= 'init' (identifier '=' expr)+
inv        ::= 'inv' predicate
op         ::= 'op' name '(' params_typed ')' ('→' type_expr)? op_body
fn         ::= 'fn' name '(' params_typed ')' '→' type_expr '=' expr
prop       ::= 'prop' predicate
```

### Operation body

```
op_body    ::= (pre | post | ret)*
pre        ::= 'pre' predicate
post       ::= 'post' predicate        # use primed vars for post-state
ret        ::= 'ret' expr
```

### Type expressions

```
type_expr  ::= Name                     # named type or type variable
             | '{' identifier (',' identifier)* '}'   # enumeration set
             | type_expr '→' type_expr  # function/mapping type
             | type_expr '×' type_expr  # product (tuple)
             | 'Seq' '[' type_expr ']'  # ordered sequence
             | '℘' '(' type_expr ')'   # power set
             | '{' identifier '∈' type_expr '|' predicate '}'  # comprehension
             | '{' field_decl (',' field_decl)* '}'  # record
```

### Field declarations (records)

```
field_decl ::= identifier ':' type_expr
```

### Expressions and predicates

```
expr       ::= identifier | literal | expr '(' args ')' | expr '.' identifier
             | expr '+' expr | expr '-' expr | ...
             | '{' (expr (',' expr)*)? '}'          # set literal
             | '[' (expr (',' expr)*)? ']'          # sequence literal
             | expr '[' expr ']'                     # indexing
             | '{' identifier '↦' expr '}'          # singleton mapping
             | '|' expr '|'                          # cardinality

predicate  ::= expr '∈' expr | expr '∉' expr
             | expr '⊆' expr | expr '⊂' expr
             | predicate '∧' predicate
             | predicate '∨' predicate
             | '¬' predicate
             | predicate '⟹' predicate
             | '∀' identifier '∈' expr '·' predicate
             | '∃' identifier '∈' expr '·' predicate
             | expr '=' expr | expr '≠' expr
             | expr '<' expr | expr '≤' expr
             | expr '>' expr | expr '≥' expr
             | 'true' | 'false' | '⊤' | '⊥'
```

## State Convention

- **Unprimed** variables refer to the **pre-state** (before the operation).
- **Primed** variables (e.g. `items'`) refer to the **post-state** (after the operation).
- **Implicit frame**: any state variable NOT mentioned in `post` clauses is **unchanged**.

## Built-in Functions

| Function | Signature | Meaning |
|----------|-----------|---------|
| `dom(f)` | `(A → B) → ℘(A)` | domain of a mapping |
| `ran(f)` | `(A → B) → ℘(B)` | range of a mapping |
| `head(s)` | `Seq[T] → T` | first element |
| `tail(s)` | `Seq[T] → Seq[T]` | all but first |
| `last(s)` | `Seq[T] → T` | last element |
| `init(s)` | `Seq[T] → Seq[T]` | all but last |
| `len(s)` | `Seq[T] → ℕ` | sequence length |
| `\|S\|` | `℘(T) → ℕ` | set cardinality |
| `min(S)` | `℘(ℤ) → ℤ` | minimum element |
| `max(S)` | `℘(ℤ) → ℤ` | maximum element |

## Sequence Operations

| Syntax | Meaning |
|--------|---------|
| `s ++ t` | concatenation |
| `s[i]` | indexing (0-based) |
| `s[i..j]` | slicing |
| `[x] ++ s` | prepend |
| `s ++ [x]` | append |

## Mapping Operations

| Syntax | Meaning |
|--------|---------|
| `f(x)` | application (lookup) |
| `f ∪ {k ↦ v}` | extend/update mapping |
| `f \ {k ↦ f(k)}` | remove key |
| `dom(f)` | set of keys |
| `ran(f)` | set of values |
| `f ⊕ g` | override (`g` wins on shared keys) |

## Import Resolution

`import path/to/name` resolves to `path/to/name.alg` relative to the importing file.

- Multiple imports are allowed.
- Circular imports are forbidden.
- Imported names are available unqualified. Use `Module.name` if disambiguation is needed.

## Extends

`spec Child extends Parent` inherits all state, invariants, and operations from Parent.

- A spec may extend at most one other spec.
- Child can add new state, invariants, and operations.
- Child can override operations by redefining them with the same name.
- Parent invariants still hold in the child; child can add stricter ones.

## Pure Functions

`fn` defines a pure helper (no state mutation):

```
fn abs(x : ℤ) → ℤ = if x ≥ 0 then x else -x
```

Use `fn` for shared logic referenced in `pre`/`post`/`inv` clauses.

## Properties

`prop` states a logical property believed to hold. It is not checked — it documents intent:

```
prop ∀x ∈ dom(store) · remove(x) ⟹ x ∉ dom(store')
```
