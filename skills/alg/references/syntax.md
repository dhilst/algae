# `.alg` Language Reference

## File Format

- Extension: `.alg`
- Encoding: UTF-8
- Comments: `#` to end of line, as in bash. `fmt` preserves them verbatim,
  along with all whitespace and layout.
- Whitespace is insignificant.
- A file contains top-level algebraic declarations; there is no `spec` wrapper.
- Declarations end with `;` (a `rule` ends with `end;`).

## Keywords

```
sort  param  op  eq  prop  lemma  rule  proof  qed  wip  by  apply  case  end  st
goal  rewrite  therefore  done
include  open  with  alias  props
true  false  if  then  else  let  in
```

A declaration must start with `sort`, `param`, `op`, `eq`, `prop`, `lemma`,
`rule`, `include`, `open`, `alias`, or `let`. The older `var` and `axiom`
keywords are gone (`var` declarations are replaced by binders; `axiom` is
replaced by `eq`), as are the state-machine keywords (`spec`, `state`, `init`,
…), which are ordinary identifiers now.

## Symbols And ASCII Aliases

Unicode symbols and their ASCII aliases (keywords or symbolic spellings) are
interchangeable. The formatter emits Unicode by default and emits the aliases
below with `fmt --ascii`. Additional symbolic input spellings are accepted:
`->` for `→`, `-/->` for `⇸`, `==>` for `⟹`, `<==>` for `⟺`, `!=` for `≠`,
`&&` for `∧`, `||` for `∨`, `|-` for `⊢`, `|>` for `▷`.

| Symbol | Alias | Meaning |
|--------|-------|---------|
| `×` | `*` (or `product`) | product |
| `→` | `arrow` | operation/function/kind arrow |
| `⇸` | `-/->` | partial operation arrow |
| `𝔹` | `Bool` | booleans |
| `Prop` | `Prop` | proposition type (rule predicates) |
| `Sort` | `Sort` | the kind of sorts |
| `¬` | `not` | negation |
| `∧` | `/\` (or `and`) | conjunction |
| `∨` | `\/` (or `or`) | disjunction |
| `⟹` | `implies` | implication |
| `⟺` | `iff` | biconditional |
| `≠` | `neq` | not equal |
| `⊤` | `truth` | logical top |
| `⊥` | `falsehood` | logical bottom |
| `▷` | `\|>` | pipe-last application sugar |
| `⊢` | `\|-` | sequent turnstile (context ⊢ goal) |
| `∀` | `forall` | universal quantifier (`∀ (n : Nat) st …`) |
| `∃` | `exists` | existential quantifier |
| `λ` | `fun` | abstraction (`λ (n : Nat) => …`) |
| `=>` | `=>` | lambda body separator |
| `:=` | `:=` | assumption naming / substitutions (`h := A`, `T := Elem`) |
| `::` | `::` | namespace separator (`list::cons`) |

`st` (a keyword) separates a quantifier's binders from its body. `::` separates
the segments of a qualified name or module path.

There are **no built-in numeric sorts**: `Nat`, `Int`, `Real` are ordinary
identifiers, not aliases, and there are no number literals or built-in
arithmetic. Numbers, if needed, are user-declared (a `Nat` sort with `z`/`s`),
or live in a library module. The only built-in types are `𝔹` and `Prop`
(and `Sort`, the kind of sorts).

The box-drawing rule bar `─` (one or more, e.g. `─────`) separates a rule's
premises from its conclusion. It is not a symbol alias — write it as `─`
(U+2500); `fmt` preserves it verbatim.

The `∀`/`∃` quantifiers and `λ` abstraction exist only inside propositions —
rule premises, conclusions, and predicate arguments — not in ordinary
equational terms.

## Grammar

```
file        ::= decl*
decl        ::= sort_decl | param_decl | op_decl | eq_decl | prop_decl
              | lemma_decl | rule_decl | include_decl | open_decl
              | alias_decl | let_decl

sort_decl   ::= 'sort' identifier ':' kind ';'      # sort Nat : Sort;  List : Sort → Sort;
param_decl  ::= 'param' identifier ':' kind ';'     # param T : Sort;
kind        ::= 'Sort' ('→' 'Sort')*                # Sort, Sort → Sort, Sort → Sort → Sort

op_decl     ::= 'op' identifier ':' domain op_arrow type_expr ';'
op_arrow    ::= '→' | '⇸'         # ⇸ declares a partial operation
domain      ::=                    # empty domain for nullary operations (constants)
              | type_product
              | type_product ('|' type_product)+   # one sum-typed argument

eq_decl     ::= 'eq'   decl_name binders? equation ';'   # a trusted equation
prop_decl   ::= 'prop' decl_name binders? equation ';'   # an instantiation obligation
lemma_decl  ::= 'lemma' decl_name binders? equation ';' proof_block?
decl_name   ::= identifier "'"*
equation    ::= expr                  # an equation lhs = rhs (not a sequent, no forall)

binders     ::= '(' binder_entry (',' binder_entry)* ')'   # ( a : A, b b' : B )
binder_entry::= identifier+ ':' type_expr                  # co-typed names share a type

rule_decl   ::= 'rule' identifier binders rule_premise* RULE_BAR prop 'end' ';'
rule_premise::= 'case' identifier prop 'end' ';'           # a named premise

prop        ::= expr | sequent
sequent     ::= context? '⊢' expr
context     ::= context_entry (',' context_entry)*
context_entry ::= identifier ':' type_expr                 # a typed context variable
              | identifier ':=' expr                       # a named assumption
              | expr                                       # an unnamed assumption

include_decl ::= 'include' module_path with_clause? obligation_block? ';'
with_clause  ::= 'with' '(' with_binding (',' with_binding)* ')'
with_binding ::= identifier ':=' type_expr     # param := sort, or op := op-name
obligation_block ::= 'props' case_block* terminator   # discharge each prop; a subproof
open_decl    ::= 'open' module_path '(' identifier (',' identifier)* ')' ';'
alias_decl   ::= 'alias' identifier '=' module_path ';'
module_path  ::= identifier ('::' identifier)*

proof_block ::= 'proof' proof_step* terminator ';'
proof_step  ::= 'goal' prop 'by' simple_tactic 'therefore' (prop | 'done') ';'
              | 'goal' prop 'by' apply case_block* 'therefore' (prop | 'done') terminator ';'
simple_tactic ::= 'rewrite' ('>' | '<') theorem 'with' '(' expr ':=' expr ')'
              | 'wip'                          # leave the goal work-in-progress (viral)
apply       ::= 'apply' identifier '(' args? ')'   # cases follow; terminator after `therefore`
terminator  ::= 'qed' | 'wip'                  # `wip` closes work-in-progress subproofs
theorem     ::= decl_name ('(' args? ')')?     # an eq/lemma instance or a local assumption
case_block  ::= 'case' identifier proof_step* terminator ';'

let_decl    ::= 'let' identifier '=' expr ';'
```

`binders` is the one binder-list form used everywhere (quantifiers, `λ`, rule
parameters, eq/prop/lemma parameters): a parenthesised, comma-separated list of
entries, where an entry may give several space-separated names one shared
type — `(a : A, b b' : B, c : C)`.

A top-level `|` in an op domain folds the whole domain into a single
sum-typed argument, grouping as in codomains (`×` binds tighter than `|`):
`op assert : Stack × Elem | Error ⇸ Stack × Elem;` takes one argument of
type `(Stack × Elem) | Error`. An arrow inside a domain branch needs parens.

## Sorts, Kinds, And Parameters

A **sort** is declared with an explicit kind. `Sort` is the kind of ordinary
sorts; `Sort → Sort` is the kind of a unary sort constructor, and so on:

```
sort Nat  : Sort;
sort List : Sort → Sort;
sort Pair : Sort → Sort → Sort;
```

A sort constructor is applied with `[...]`: `List[Nat]`, `Pair[A, B]`,
`list::List[Elem]`. A use must supply the right number of type arguments.

A **parameter** is an abstract sort or sort constructor that a module exports
for instantiation:

```
param T : Sort;
param M : Sort → Sort;
```

`param` names are valid on the left of `include … with (…)` substitutions
(`T := Nat`). Within the module they behave like opaque sorts.

## Type Expressions

```
type_expr    ::= type_sum
type_sum     ::= type_arrow ('|' type_arrow)*       # algebraic sum/union type
type_arrow   ::= type_product (('→' | '⇸') type_arrow)?
type_product ::= type_primary ('×' type_primary)*
type_primary ::= type_name | '𝔹' | 'Prop' | 'Sort'
               | 'Seq' '[' type_expr ']'
               | '(' type_expr ')'
               | '()'
type_name    ::= module_path ('[' type_expr (',' type_expr)* ']')?   # List[T], list::List[Elem]
```

`Prop` is the type of propositions, built in (not a sort), used for rule
predicate parameters such as `P : Nat → Prop`. `𝔹` is the boolean type used by
`=`, the connectives, and `if`/`then`/`else`. `Sort` is the kind written in
`sort`/`param` declarations and in rule parameters such as `(T : Sort, x : T)`.

## Equations: `eq`, `prop`, And `lemma`

`eq`, `prop`, and `lemma` share one shape — a name, optional binders, and an
equation body `lhs = rhs`. The binder variables are the equation's schematic
parameters. There are no top-level variables; every variable used in a body
must be introduced by the binders.

```
eq   add_zero_left(n : Nat) add(z, n) = n;          # a trusted equation
prop left_identity(x : T)   mul(unit, x) = x;       # an obligation (see Modules)
lemma add_zero_right(n : Nat) add(n, z) = n;        # provable (see Proofs)
```

The three differ in trust, not shape:

- **`eq`** is a trusted equation, available as a rewrite justification.
- **`prop`** is a proof obligation: not trusted, and discharged at the
  `include` site that instantiates the module declaring it.
- **`lemma`** is proved locally by an optional `proof … qed;` block; a proven
  equational lemma may be used as a rewrite justification like an `eq`.

A name is an identifier with trailing primes allowed (`eq assoc' …;`), and is
required to be unique across the module (`eq`, `prop`, `lemma`, and `rule`
names share one namespace). An eq/prop/lemma body is an equation — **not** a
sequent and **not** written with `forall`.

Instantiating an equation substitutes its binders; e.g. `add_zero_left(z)`
produces `add(z, z) = z`.

## Terms

```
expr      ::= identifier | qualified_name
            | 'true' | 'false' | '⊤' | '⊥'
            | expr '(' args ')'
            | expr "'"                           # prime: a primed term (e.g. x', f(a)')
            | '(' expr ')' | '(' ')' | '(' expr (',' expr)+ ')'   # unit / tuple
            | expr ('=' | '≠') expr
            | expr ('∧' | '∨' | '⟹' | '⟺') expr
            | '¬' expr
            | expr '++' expr                     # sequence concatenation
            | expr '.' expr                      # pipe-first application sugar
            | expr '▷' expr                      # pipe-last application sugar
            | 'if' expr 'then' expr 'else' expr
            | 'let' let_lhs '=' expr 'in' expr
            | 'λ' binders '=>' expr             # abstraction: λ (a : A, b : B) => …
            | ('∀' | '∃') binders 'st' expr     # quantifier:  ∀ (a : A, b : B) st …

qualified_name ::= identifier ('::' identifier)*   # list::cons
let_lhs    ::= identifier
             | '(' binder ',' binder (',' binder)* ')'   # destructuring
binder     ::= identifier | '_'
```

A **nullary op is a constant**, used bare (`z`, `nil`, `empty_error`); it may
also be written with empty parentheses (`empty()`). Non-nullary ops are
applied (`s(n)`, `add(z, n)`).

`λ`, `∀`, and `∃` extend greedily to the right (like `if`/`let`); in an operand
position they are parenthesized. A `λ` body that is a proposition gives the
abstraction codomain `Prop`, so `λ (n : Nat) => add(n, z) = n` has type
`Nat → Prop`; a multi-binder `λ (a : A, b : B) => …` takes a product domain
`A × B`, applied as `f(a, b)`. Connectives (`∧ ∨ ⟹ ⟺ ¬`) accept both `𝔹` and
`Prop` operands.

A trailing `'` (prime) is a postfix on any term: `x'`, `s.pop'`, `f(a)'`. A
primed term has the same type as the term it follows (a naming convention, e.g.
"the next state"); it carries no extra checking. Binder names may also carry
primes (`(a b' : T)`).

`let … in` names an intermediate term so deeply nested equations stay readable;
chains break the line after each `in`. A top-level `let name = expr;` (no `in`)
names a term once for the declarations that follow.

### Destructuring Let

A `let ... in` pattern with two or more binders takes a product-typed value
apart, naming its components:

```
op get : Queue × Elem → Queue × Elem;

eq head_get(q : Queue, d : Elem) q.head(d) = let (_, x) = q.get(d) in x;
```

`_` is a binder for components the body does not use; **each `_` is a distinct
fresh variable**, and `_` is not valid anywhere else. Destructuring requires the
value's type to be a product with exactly as many components as the pattern has
binders; a sum cannot be destructured.

## Partial Operations

An op declared with `⇸` (ASCII `-/->`) is **partial**: applying it carries a
proof obligation the spec does not discharge mechanically. The intended use is
narrowing a sum to one of its branches:

```
op pop    : Stack → Stack × Elem | Error;
op assert : Stack × Elem | Error ⇸ Stack × Elem;

eq assert_elim(s : Stack, e : Elem) (s, e).assert = (s, e);
```

For now `⇸` is purely syntactic — `check` treats a partial op exactly like a
total one and does not validate applicability.

## Propositions, Rules, And Proofs

A **proposition** is either a plain boolean `expr` or a **sequent** of the form
`context ⊢ goal`. A context entry is a typed variable (`n : Nat`), a named
assumption (`h := A`), or an unnamed assumption (`A`).

A **rule** declares a named inference rule: typed parameters, zero or more
**named premise cases**, a rule bar `─────`, a conclusion proposition, and
`end;`. A `T → Prop` parameter is a predicate over `T`. Premise contexts may
carry typed variables, but premise assumptions are left **unnamed** — names are
introduced only in the proof's `case`:

```
rule reflexivity(T : Sort, x : T)
  ─────────────────────────
  ⊢ x = x
end;

rule induction(P : Nat → Prop)
  case base
    ⊢ P(z)
  end;
  case step
    n : Nat, P(n) ⊢ P(s(n))
  end;
  ─────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;
```

A **proof** is a sequence of proof steps, each transforming an input proof
state into an output state:

```
goal
  <input proof state>
by <tactic>
therefore
  <output proof state | done>;
```

`goal` opens a step; `therefore` closes it. `done` means the tactic discharged
the goal with no remaining subgoals. There is no `goal` keyword after
`therefore`.

Three tactics:

- **`rewrite > theorem(args) with ( lhs := rhs )`** rewrites left-to-right
  (`<` for right-to-left). The `theorem` is an `eq`/`lemma` instance or a local
  assumption (e.g. `ih`); the substitution `( lhs := rhs )` names the exact
  subterm replacement, in **parentheses**.
- **`apply rule(args) <cases> therefore <result> qed;`** applies an inference
  rule. Each predicate argument is given explicitly as a `λ`. The cases follow
  the application and are matched to the rule's premises **by name**; the `apply`
  is itself a subproof, so its `qed`/`wip` terminator comes **after** the
  step's `therefore <result>`. A zero-premise rule has no cases —
  `by apply reflexivity(Nat, z) therefore done qed;`.
- **`wip`** ("work in progress") discharges the goal provisionally, to be
  finished later. It is **viral**: see *Work-in-progress proofs* below.

A subproof — a `proof` block, a `case`, an `apply`, or an include `props` block —
is closed by a **terminator**, either `qed` or `wip`.

### Work-in-progress proofs

`wip` closes any subproof that uses the `wip` tactic, directly or through a
contained subproof that does. The marker is viral: a `case` discharged by `wip`
is closed with `wip`; that marks the enclosing `apply` work-in-progress, which
must also be `wip`; and so on up to the `proof` block. `qed` is only allowed to
close a subproof with no `wip` content — closing a work-in-progress subproof
with `qed` is an error. (Verification is left to a later phase; for now the
checker only tracks the marker.)

The full induction proof of `add_zero_right`:

```
lemma add_zero_right(n : Nat)
  add(n, z) = n;
proof
  goal
    ⊢ add(n, z) = n
  by apply induction(λ (n : Nat) => add(n, z) = n)
    case base
      goal
        ⊢ add(z, z) = z
      by rewrite > add_zero_left(z) with (add(z, z) := z)
      therefore
        ⊢ z = z;
      goal
        ⊢ z = z
      by apply reflexivity(Nat, z)
      therefore done
      qed;
    qed;
    case step
      goal
        n : Nat, ih := add(n, z) = n ⊢ add(s(n), z) = s(n)
      by rewrite > add_succ_left(n, z) with (add(s(n), z) := s(add(n, z)))
      therefore
        n : Nat, ih := add(n, z) = n ⊢ s(add(n, z)) = s(n);
      goal
        n : Nat, ih := add(n, z) = n ⊢ s(add(n, z)) = s(n)
      by rewrite > ih with (add(n, z) := n)
      therefore
        n : Nat, ih := add(n, z) = n ⊢ s(n) = s(n);
      goal
        n : Nat, ih := add(n, z) = n ⊢ s(n) = s(n)
      by apply reflexivity(Nat, s(n))
      therefore done
      qed;
    qed;
  therefore done
  qed;
qed;
```

Proofs are **structure-checked, not discharged**: the checker validates that the
rule exists, the argument count and types match, and the provided `case` names
exactly cover the rule's premise names. It does **not** verify that a tactic
actually transforms the input goal into the output goal — rewrite steps are
recorded but never applied.

## Modules

Specifications can be split across files. A **project** is rooted at the nearest
ancestor directory containing `alg-project.json`:

```json
{ "include_path": ["."], "vendor": "vendor" }
```

A module path `foo::bar` resolves to `foo/bar.alg`, searched across the
`include_path` directories and then the `vendor/` directory (both relative to
the project root). A file that uses no `include`/`open` needs no project file.

- `include foo::bar with (T := Elem);` brings the module's declarations in under
  the qualified namespace `foo::bar` (`foo::bar::cons`). `with` instantiates the
  module's parameters: a `param` LHS takes a sort, an op LHS takes an op name;
  omitting a binding leaves a parameter abstract.
- `alias bar = foo::bar;` shortens a namespace, so `bar::cons` means
  `foo::bar::cons`.
- `open foo::bar (nil, cons);` additionally exposes the named declarations
  unqualified. The name list is required, and `open` needs a prior `include`.

```
# list.alg
param T : Sort;
sort List : Sort → Sort;
op nil  : → List[T];
op cons : T × List[T] → List[T];

# use_list.alg
include list with (T := Elem);
alias l = list;
open list (nil, cons);
sort Elem : Sort;
eq built(e : Elem, xs : list::List[Elem]) cons(e, xs) = l::cons(e, xs);
```

### Obligations

When an included module declares `prop`s, each becomes an **obligation** the
include must discharge in a `props … <terminator>` block, with one matching
`case <prop-name>` per `prop`:

```
# monoid.alg
param T : Sort;
op unit : → T;
op mul  : T × T → T;
prop left_identity(x : T)  mul(unit, x) = x;
prop associativity(x y z : T)  mul(mul(x, y), z) = mul(x, mul(y, z));

# nat_monoid.alg
include nat;
open nat (z, s, add);
include monoid with (T := nat::Nat, unit := z, mul := add) props
  case left_identity
    goal
      ⊢ add(z, x) = x
    by rewrite > add_zero_left(x) with (add(z, x) := x)
    therefore done;
  qed;
  case associativity
    # the case shape is required; the proof body may be elided
  qed;
qed;
```

`check` resolves includes (with `with`-substitution applied), validates each
included module on its own, type-checks qualified and opened references, and
requires one obligation `case` per included `prop`. Includes are resolved
transitively with cycle detection. `fmt` and `print` operate on a single file.

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
eq push_pop(s : Stack, e : Elem) s.push(e).pop = (s, e);   # pop(push(s, e))
```

`.` binds tightly (above `++`, below calls); `▷` binds loosely (above `++`,
below the comparisons). The reading is defined only when the right operand is a
call or a bare name; anything else is rejected by the type checker.

Match the spec's style to its implementation target and keep one style per
spec: data-first (`.`) for object-oriented targets where the structure is the
receiver, data-last (`▷`) for functional targets where structure-last
signatures curry into pipelines.

## Type Checking

`check` parses and then type-checks. The rules:

- Every identifier in a body must resolve: a binder, a local `let`, a top-level
  `let`, or an op (a nullary op resolves to a constant of its codomain). A
  built-in type used in *term* position (`𝔹`, `Prop`, `Sort`) is a type error
  (`… is a sort, not a term`).
- A `sort`/`param` kind must be `Sort` or `Sort → … → Sort`. A sort use must
  supply as many type arguments as its kind's arity (`List[Elem]`).
- Ops may be **overloaded**: the same name with different domains is resolved
  by argument types. Unresolvable or ambiguous calls are errors.
- Sums **inject**: a term of type `T` is accepted where `T | Error` is
  expected. Sums do **not** implicitly narrow: a term of type `T | Error` is
  rejected where `T` is expected. To assert the happy path, declare an explicit
  cast op — by convention `op cast : (T | Error) → T;` — and wrap the fallible
  term. A sum-typed value can never be destructured.
- Equations `=`/`≠` require compatible operand types (either direction) and
  yield `𝔹`; `++` requires matching `Seq` operands. Connectives (`∧ ∨ ⟹ ⟺ ¬`)
  accept `𝔹` or `Prop` operands and yield `Prop` when either operand is a
  `Prop`, else `𝔹`.
- An eq/prop/lemma body must type to a proposition (`𝔹` or `Prop`); in a
  sequent, every assumption and the goal must. Explicit `(binders)` are checked
  in scope.
- `eq`, `prop`, `lemma`, and `rule` names share one namespace and must be
  unique. A rule's parameters, premises, and conclusion are type-checked, and a
  premise assumption may not be named.
- Proofs are structure-checked: `apply` checks the rule exists, the argument
  count and types match, and the case names cover the premise names; rewrite
  steps are recorded but not discharged. A subproof that uses the `wip` tactic
  (directly or virally) must be closed with `wip`, not `qed`. An `include` of a
  module with `prop`s discharges them in a `props … <terminator>` block with one
  `case` per `prop`.

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
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;

op empty : → Stack;
op push  : Stack × Elem → Stack;
op pop   : Stack → Stack × Elem | Error;
op top   : Stack → Elem | Error;

eq push_pop(s : Stack, e : Elem) s.push(e).pop = (s, e);
eq push_top(s : Stack, e : Elem) s.push(e).top = e;
eq empty_pop empty().pop = empty_error;
eq empty_top empty().top = empty_error;
```
