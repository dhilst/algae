# `.alg` Language Reference

## File Format

- Extension: `.alg`
- Encoding: UTF-8
- Comments: `#` to end of line, as in bash. `fmt` preserves them verbatim,
  along with all whitespace and layout.
- Whitespace is insignificant.
- A file contains top-level algebraic declarations; there is no `spec` wrapper.
- Declarations end with `;`.

## Keywords

```
sort  op  var  axiom  lemma  rule  proof  qed  by  apply  case  end  st
include  open  with  alias
true  false  if  then  else  let  in
```

The previous state-machine syntax is not part of this grammar. Its keywords
(`spec`, `state`, `init`, `pre`, `post`, ...) are ordinary identifiers now, but
old-syntax files still fail to parse since declarations must start with `sort`,
`op`, `var`, `axiom`, `lemma`, `rule`, `include`, `open`, `alias`, or `let`.

## Symbols And ASCII Aliases

Unicode symbols and their ASCII aliases (keywords or symbolic spellings) are
interchangeable. The formatter emits Unicode by default and emits the aliases
below with `fmt --ascii`. Additional symbolic input spellings are accepted:
`->` for `→`, `==>` for `⟹`, `<==>` for `⟺`, `!=` for `≠`, `<=` for `≤`,
`>=` for `≥`, `&&` for `∧`, `||` for `∨`, `|-` for `⊢`.

| Symbol | Alias | Meaning |
|--------|-------|---------|
| `×` | `*` (or `product`) | product |
| `→` | `arrow` | operation/function arrow |
| `⇸` | `-/->` | partial operation arrow |
| `ℕ` | `Nat` | natural numbers |
| `ℤ` | `Int` | integers |
| `ℝ` | `Real` | reals |
| `𝔹` | `Bool` | booleans |
| `Prop` | `Prop` | proposition type (rule predicates) |
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
| `⊢` | `\|-` | sequent turnstile (assumptions ⊢ goal) |
| `∀` | `forall` | universal quantifier (`∀ (n : ℕ) st …`) |
| `∃` | `exists` | existential quantifier |
| `λ` | `fun` | abstraction (`λ (n : ℕ) => …`) |
| `=>` | `=>` | lambda body separator |
| `:=` | `:=` | assumption naming / `with` binding (`h := A`, `T := Elem`) |
| `::` | `::` | namespace separator (`list::cons`) |

`st` (a keyword) separates a quantifier's binders from its body. `::` separates
the segments of a qualified name or module path.

The box-drawing rule bar `─` (one or more, e.g. `─────`) separates a rule's
premises from its conclusion. It is not a symbol alias — write it as `─`
(U+2500); `fmt` preserves it verbatim.

Set-theory notation (`∈`, `⊆`, `∪`, `∩`, `∅`, `℘`, set and mapping literals) is
not part of the grammar. Specifications are equational: behavior is captured by
axioms over constructor terms, with `var` declarations read as implicitly
universally quantified. The `∀`/`∃` quantifiers and `λ` abstraction exist only
inside propositions — rule premises, conclusions, and predicate arguments — not
in ordinary equational terms.

`empty` and `top` are ordinary identifiers so they can name operations.

## Grammar

```
file        ::= decl*
decl        ::= sort_decl | op_decl | var_decl | axiom_decl | lemma_decl
              | rule_decl | include_decl | open_decl | alias_decl | let_decl

sort_decl   ::= 'sort' identifier (',' identifier)* ';'
              | 'sort' identifier '[' identifier (',' identifier)* ']' ';'  # parametric
              | 'sort' identifier '=' '{' identifier (',' identifier)* '}' ';'

op_decl     ::= 'op' identifier ':' domain op_arrow type_expr ';'
op_arrow    ::= '→' | '⇸'         # ⇸ declares a partial operation
domain      ::=                    # empty domain for nullary operations
              | type_product
              | type_product ('|' type_product)+   # one sum-typed argument

var_decl    ::= 'var' identifier (',' identifier)* ':' type_expr ';'
axiom_decl  ::= 'axiom' rule_name binders? prop ';'
              | 'axiom' rule_name '=' prop ';'
rule_name   ::= identifier "'"*
lemma_decl  ::= 'lemma' rule_name binders? prop ';' proof_block?
              | 'lemma' rule_name '=' prop ';' proof_block?

binders     ::= '(' binder_entry (',' binder_entry)* ')'   # ( a : A, b b' : B )
binder_entry::= identifier+ ':' type_expr                  # co-typed names share a type

prop        ::= expr | sequent              # a proposition
sequent     ::= assumptions? '⊢' expr
assumptions ::= assumption (',' assumption)*
assumption  ::= expr | identifier ':=' expr  # an optionally-named hypothesis

rule_decl   ::= 'rule' identifier binders
                prop*                        # premises
                RULE_BAR                     # one or more '─'
                prop                         # conclusion
                'end'

include_decl ::= 'include' module_path ('with' '(' with_binding (',' with_binding)* ')')? ';'
open_decl    ::= 'open' module_path '(' identifier (',' identifier)* ')' ';'
alias_decl   ::= 'alias' identifier '=' module_path ';'
module_path  ::= identifier ('::' identifier)*
with_binding ::= identifier ':=' type_expr

proof_block ::= 'proof' proof_step* 'qed' ';'
proof_step  ::= expr ';'
              | '=' expr 'by' rule_name ';'
              | apply_step
apply_step  ::= 'apply' rule_name '(' args? ')' ';' case_block+
case_block  ::= 'case' '[' case_sequent ']' proof_step* 'qed' ';'
case_sequent::= (identifier ':=' expr (',' …)*)? '⊢' expr   # the branch's subgoal

let_decl    ::= 'let' identifier '=' expr ';'
```

`binders` is the one binder-list form used everywhere (quantifiers, `λ`, rule
parameters, axiom/lemma parameters): a single parenthesised, comma-separated
list of entries, where an entry may give several space-separated names one
shared type — `(a : A, b b' : B, c : C)`.

A top-level `|` in an op domain folds the whole domain into a single
sum-typed argument, grouping as in codomains (`×` binds tighter than `|`):
`op assert : Stack × Elem | Error ⇸ Stack × Elem;` takes one argument of
type `(Stack × Elem) | Error`. An arrow inside a domain branch needs parens.

## Type Expressions

```
type_expr    ::= type_sum
type_sum     ::= type_arrow ('|' type_arrow)*       # algebraic sum/union type
type_arrow   ::= type_product (('→' | '⇸') type_arrow)?
type_product ::= type_primary ('×' type_primary)*
type_primary ::= type_name | 'ℕ' | 'ℤ' | 'ℝ' | '𝔹' | 'Prop'
               | 'Seq' '[' type_expr ']'
               | '(' type_expr ')'
               | '()'
type_name    ::= module_path ('[' type_expr (',' type_expr)* ']')?   # List[T], list::List[Elem]
```

`Prop` is the type of propositions. It is built in (not a sort) and exists only
for rule parameters such as `P : ℕ → Prop`; it does not participate in
algebraic specifications.

A **parametric sort** `sort List[T];` introduces a type constructor of arity 1.
Its parameters are module-wide type variables, in scope for every signature
(`op cons : T → List[T] → List[T];`). A use must supply the right number of
arguments: `List[Elem]`, `list::List[Elem]`. A name may be qualified with a
module path (`list::List`) once the module is included.

## Terms And Axioms

`check` type-checks declarations and axioms (see Type Checking below); axioms
are not proved or model-checked. `var` declarations are read as implicitly
universally quantified over all axioms. A multi-name declaration
(`var e, f : Elem;`) declares every name at the same sort and is kept grouped
by `fmt`.

Every axiom carries a required name — an identifier with trailing primes
allowed (`axiom empty_size q.empty ⟺ q.size = 0;`, `axiom assoc' …;`) —
which `check` requires to be unique across the module (axiom, lemma, and rule
names share one namespace).

An axiom (and a lemma) is a quantified proposition. Three equivalent forms:

```
axiom f foo = foo;                        # foo's type comes from `var foo : T;`
axiom f (foo : T) foo = foo;              # explicit binder
axiom f = forall (foo : T) st foo = foo;  # the proposition written out
```

The first identifier after `axiom` is always the name. The body then starts
either at `=` (the proposition form), at a `( name … : type )` binder list, or
directly (free variables resolve to declared `var`s). A body that begins with a
parenthesised expression — `axiom refl' (q, q) = (q, q');` — is a term, not a
binder list, because no `:` follows the names.

```
expr      ::= identifier | qualified_name
            | literal
            | expr '(' args ')'
            | '(' expr ')'
            | expr comparison expr
            | expr bool_op expr
            | expr '.' expr
            | expr '▷' expr
            | 'if' expr 'then' expr 'else' expr
            | 'let' let_lhs '=' expr 'in' expr
            | 'λ' binders '=>' expr             # abstraction: λ (a : A, b : B) => …
            | ('∀' | '∃') binders 'st' expr     # quantifier:  ∀ (a : A, b : B) st …

qualified_name ::= identifier ('::' identifier)*   # list::cons

let_lhs    ::= identifier
             | '(' binder ',' binder (',' binder)* ')'   # destructuring
binder     ::= identifier | '_'

comparison ::= '=' | '≠' | '<' | '≤' | '>' | '≥'
bool_op    ::= '∧' | '∨' | '⟹' | '⟺'
```

`λ`, `∀`, and `∃` extend greedily to the right (like `if`/`let`); in an operand
position they are parenthesized. A `λ` body that is a proposition gives the
abstraction codomain `Prop`, so `λ (n : ℕ) => n + 0 = n` has type `ℕ → Prop`; a
multi-binder `λ (a : A, b : B) => …` takes a product domain `A × B`, applied as
`f(a, b)`. Connectives (`∧ ∨ ⟹ ⟺ ¬`) accept both `𝔹` and `Prop` operands.

`let` names an intermediate term so deeply nested axioms stay readable. Lets
nest, so a chain of bindings conventionally breaks the line after each `in`
with the bindings aligned:

```
axiom authorized_happy let with_user = add_user(rbac, u) in
      let with_role = add_role(with_user, r) in
      authorized(with_role, u, p) = true;
```

A `let` may also appear at top level (no `in`), naming a term once for every
axiom that follows. This avoids repeating a common setup chain:

```
let with_user = add_user(empty_rbac(), u);
let with_role = add_role(with_user, r);

axiom no_roles authorized(with_role, u, p) = false;
axiom removed let revoked = remove_user(with_role, u) in authorized(revoked, u, p) = unknown_user;
```

Top-level lets are abbreviations: the parser records them but does not check
that axioms reference them, and variables inside the named term are still read
as universally quantified per axiom.

### Destructuring Let

A `let ... in` pattern with two or more binders takes a product-typed value
apart, naming its components:

```
op pop : NEStack → Stack × Elem;

axiom pop_size let (rest, top) = pop(n) in size(rest) = size(n) - 1;
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

## Partial Operations And Lemmas

An op declared with `⇸` (ASCII `-/->`) is **partial**: applying it carries a
proof obligation the spec does not discharge mechanically yet. The intended
use is narrowing a sum to one of its branches:

```
op pop    : Stack → Stack × Elem | Error;
op assert : Stack × Elem | Error ⇸ Stack × Elem;

axiom assert_elim (s, e).assert = (s, e);
```

For now `⇸` is purely syntactic — `check` treats a partial op exactly like a
total one and does not validate applicability.

A `lemma` records a derived fact, optionally with a proof sketch — a start
term followed by rewrite steps that each cite an axiom (or lemma) name:

```
lemma pop_top
  s.push(e).pop.assert.snd = e;
proof
  s.push(e).pop.assert.snd;
  = (s, e).assert.snd by push_pop;
  = (s, e).snd by assert_elim;
  = e by snd_pair;
qed;
```

Lemma names are required, like axiom names. A lemma's proposition **is**
type-checked (it must be a proposition — `𝔹` or `Prop`). The proof's rewrite
steps are still parsed and preserved but not verified: rewrite terms are not
checked and `by` references are not resolved. Full proof verification is a
future phase.

## Propositions, Rules, And Proof Branches

A **proposition** is either a plain boolean `expr` or a **sequent** of the form
`assumptions ⊢ goal`. An assumption is a boolean expression. In a `case` subgoal
it is named with `h := A` (the hypothesis a proof step may cite by name); in a
rule premise it is left **unnamed** — names are introduced only at the `case`.
Axioms, lemmas, and rules all state propositions:

```
axiom add_zero_left ⊢ 0 + m = m;
axiom add_succ_left ⊢ s(n) + m = s(n + m);
```

A **rule** declares a named inference rule: typed parameters, zero or more
premise propositions (with **unnamed** assumptions), a rule bar `─────`, a
conclusion proposition, and `end`. A `T → Prop` parameter is a predicate over
`T`:

```
rule induction(x : ℕ, P : ℕ → Prop)
  ⊢ P(0)
  P(x) ⊢ P(s(x))
  ─────────────────────
  ⊢ ∀ (n : ℕ) st P(n)
end
```

Inside a proof, `apply` invokes a rule and opens one `case` per premise. Each
predicate argument is given **explicitly** as a `λ` abstraction (nothing is
inferred). A `case` writes its branch's **full sequent** explicitly — the named
hypotheses and the goal — so the proof state is visible in the source. A premise
with no hypotheses is discharged with `case [⊢ goal]`:

```
lemma add_zero_right ⊢ n + 0 = n;
proof
  apply induction(n, λ (n : ℕ) => n + 0 = n);

  case [⊢ 0 + 0 = 0]
    0 + 0;
    = 0 by add_zero_left;
  qed;

  case [ih := n + 0 = n ⊢ s(n) + 0 = s(n)]
    s(n) + 0;
    = s(n + 0) by add_succ_left;
    = s(n) by ih;
  qed;
qed;
```

Rule names share one namespace with axiom and lemma names and must be unique.
Applying a rule **computes** each branch's subgoal by substituting the arguments
into the premise and β-reducing the predicate applications, and **verifies** the
written `case` sequent against it: the rule above computes `⊢ 0 + 0 = 0` and
`n + 0 = n ⊢ s(n) + 0 = s(n)`, which the two cases must state (the author chooses
the hypothesis names). The checker validates that the rule exists, the argument
count and types match the parameters, the number of cases equals the number of
premises, and each written subgoal — every hypothesis proposition and the goal —
matches the computed one. The proof steps within a case are not otherwise
verified.

## Modules

Specifications can be split across files. A **project** is rooted at the nearest
ancestor directory containing `alg-project.json`:

```json
{ "include_path": ["."], "vendor": "vendor" }
```

A module path `foo::bar` resolves to `foo/bar.alg`, searched across the
`include_path` directories and then the `vendor/` directory (both relative to
the project root). `std` is reserved for the future vendored standard library. A
file that uses no `include`/`open` needs no project file.

- `include foo::bar with (T := Elem);` brings the module's declarations in under
  the qualified namespace `foo::bar` (`foo::bar::cons`). `with` instantiates the
  module's type parameters; omitting it leaves them as abstract type variables.
- `alias bar = foo::bar;` shortens a namespace, so `bar::cons` means
  `foo::bar::cons`.
- `open foo::bar (nil, cons);` additionally exposes the named declarations
  unqualified. The name list is required, and `open` needs a prior `include`.

```
# list.alg
sort List[T];
op nil  : → List[T];
op cons : T → List[T] → List[T];

# use_list.alg
include list with (T := Elem);
alias l = list;
open list (nil, cons);
sort Elem;
var e : Elem;
var xs : list::List[Elem];
axiom built cons(e)(xs) = l::cons(e)(xs);
```

`check` resolves includes (with `with`-substitution applied), validates each
included module on its own, and type-checks qualified and opened references
against the imported signatures. Includes are resolved transitively with
cycle detection. `fmt` and `print` operate on a single file and do not load
included modules.

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
axiom push_pop s.push(e).pop = (s, e);             # pop(push(s, e))
axiom push_pop' s |> push(e) |> pop = (e, s);      # with data-last signatures
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

axiom push_pop s.push(e).pop = (s, e);   # pop(push(s, e))
axiom empty_pop empty().pop = empty_error;
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

axiom push_pop s ▷ push(e) ▷ pop = (e, s);  # pop(push(e, s))
axiom empty_pop empty() ▷ pop = empty_error;
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
- Comparisons yield `𝔹`; `< ≤ > ≥` and arithmetic require numerics; `++`
  requires matching `Seq` operands; equations `=`/`≠` require compatible operand
  types (either direction). Connectives (`∧ ∨ ⟹ ⟺ ¬`) accept `𝔹` or `Prop`
  operands and yield `Prop` when either operand is a `Prop`, else `𝔹`.
- A **parametric sort** `sort List[T];` has an arity; uses must supply that many
  type arguments (`List[Elem]`), and its parameters are module-wide type
  variables. A **qualified** name `mod::name` (and an `alias`) resolves against
  an included module's namespaced declarations.
- An axiom/lemma proposition must type to `𝔹` or `Prop`; in a sequent, every
  assumption and the goal must. Explicit `(binders)` are checked in scope; the
  three axiom/lemma forms are equivalent.
- Axiom, lemma, and rule names share one namespace and must be unique.
- A rule's parameters, premises, and conclusion are type-checked, and a premise
  assumption may not be named. Applying a rule checks the argument count and
  types, the case count against the number of premises, and each written `case`
  sequent against the subgoal computed from the premise (every hypothesis
  proposition and the goal must match; hypothesis names are the author's
  choice). Rewrite steps and `by` references inside proofs are parsed only, not
  verified.

Errors are reported as `<file>: type error at line <N>, <message>` with the
declaration's line. `check --syntax-only` skips type checking.

## CLI

```
algae.py check [--syntax-only] file.alg [file2.alg ...]
algae.py fmt [--ascii --inplace] file.alg [file2.alg ...]
algae.py print file.alg [file2.alg ...]
```

- `check` prints `<file>.alg: ok`, a parse error
  (`<file>.alg: error at <line>, Expected <foo> found <bar>`), or type errors
  (`<file>.alg: type error at line <N>, <message>`). `--syntax-only` skips
  type checking.
- `fmt` preserves the source verbatim — whitespace, layout, and comments —
  and only respells symbol aliases: to Unicode by default, to the canonical
  ASCII aliases with `--ascii`. `--inplace` rewrites the files.
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

axiom push_pop s.push(e).pop = (s, e);
axiom push_top s.push(e).top = e;
axiom empty_pop empty().pop = empty_error;
axiom empty_top empty().top = empty_error;
```
