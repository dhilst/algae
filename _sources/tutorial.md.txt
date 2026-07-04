# Algae Tutorial

Algae is a small proof and algebraic-specification language. You declare a
vocabulary (sorts, operators), assert facts (axioms) and inference rules, and then
prove lemmas by writing **explicit proof trees** that a tiny trusted kernel
re-checks. This tutorial builds up from a one-line proof to a proof by induction.

This document is the gentle path in. Rather than invent toy vocabulary, it works
through the **standard library** — `core`, `nat`, `option`, `group`, `monad` — so
every example is real code you can `import`. The precise reference is the language
specification (`lang-specs/spec.md` in the repository).

## Two worlds, two namespaces

Keep one idea in mind from the start — Algae has **two separate worlds**, and names
live in **two disjoint namespaces**:

- The **term world** — sorts, operators, variables, and the propositions built from
  them. This is the *term namespace*.
- The **proof world** — axioms, rules, lemmas, and hypotheses: the things you apply
  to build a proof. This is the *proof namespace*.

A name in one namespace is invisible to the other. You cannot mention a lemma inside
a proposition, and you cannot apply an operator as a proof step. We will make this
concrete in [The two namespaces](#the-two-namespaces) — but it explains a lot of the
language's shape, so it is worth holding onto now.

## Running the checker

Everything below is a `.alg` file you check with the CLI:

```sh
cargo run -p algae-cli -- verify file.alg      # elaborate + proof-check
cargo run -p algae-cli -- typecheck file.alg   # signatures only, skip proofs
cargo run -p algae-cli -- parse file.alg        # syntax only
cargo run -p algae-cli -- fmt file.alg          # normalize operator glyphs
```

`verify` is the one that runs the proof checker. A clean run prints
`… : checked N proof obligation(s)`.

### ASCII and Unicode

Every operator has an ASCII and a Unicode spelling, and both lex to the same token.
This tutorial uses the Unicode spelling; if you prefer to type ASCII, `fmt`
converts it to Unicode (and `fmt --ascii` converts back).

| ASCII | Unicode | meaning |
|-------|---------|---------|
| `\|-` | `⊢` | turnstile (sequent) |
| `->` | `→` | function type |
| `*` | `×` | product type |
| `forall` | `∀` | universal |
| `exists` | `∃` | existential |
| `lambda` | `λ` | lambda |
| `st` | `st` | "such that" (binder body separator) |
| `=>` | `⇒` | implication |
| `/\` `\/` `~` | `∧` `∨` `¬` | and, or, not |

## Sorts, operators, types

A **sort** is a base type. An **operator** is a total function symbol with a
signature. Nothing here is a proof yet — this is the term world's vocabulary. Here
is the opening of `nat.alg`:

```alg
sort Nat : Sort;            # a base sort

op 0 : → Nat;              # a nullary operator (a constant)
op s : Nat → Nat;          # successor
op + : Nat × Nat → Nat;    # a binary operator, written infix as x + y
```

Types are built from sorts with `×` (product), `|` (sum), and `→` (function). A
proposition has the special type `Prop`; a predicate is therefore an operator into
`Prop`, e.g. `op even : Nat → Prop`. `option.alg` shows all three at once — its
`bind` takes a product of an `Option(A)` and a function:

```alg
op bind : Option(A) × (A → Option(B)) → Option(B);
```

## Propositions and sequents

A **proposition** is just a `Prop`-valued term: an equation `a = b`, a connective
(`∧`, `∨`, `⇒`, `⇔`, `¬`), a quantifier (`∀`, `∃`), or a predicate
applied to arguments. Terms and propositions share one grammar.

A **sequent** is a proposition under a context of assumptions:

```
context ⊢ proposition
```

The context lists typed variables and named hypotheses. With an empty context you
just write `⊢ proposition`. These two are read "under x and y, and a proof of
x = y, conclude y = x":

```alg
⊢ x = x
x : Nat, y : Nat, h := x = y ⊢ y = x
```

A **lemma** states a sequent and must supply a proof of it.

## Axioms and definitional equality

An **axiom** asserts a sequent as true without proof. Operators get their meaning
from equational axioms; `nat.alg` gives `+` its meaning with two of them:

```alg
axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);
```

The checker's built-in notion of "the same term" — **definitional equality**
(`defeq`) — is **α/β-equivalence only**: two terms are equal when they share a
beta normal form. Operators are **inert constants**; the checker never evaluates
them, so `0 + 0` does *not* reduce to `0` on its own. An equation takes effect
only where a proof explicitly *invokes* it.

The simplest way to invoke one is to close a goal that an axiom's conclusion
already matches. Instantiating `add_zero_left` at `n = 0` makes its conclusion
`0 + n = n` read `0 + 0 = 0` — exactly the goal — so the axiom closes it outright.
Importing `nat` brings `0`, `+`, and `add_zero_left` into scope:

```alg
import nat;

lemma zero_plus_zero
  ⊢ 0 + 0 = 0;
proof
  by add_zero_left(0);
qed;
```

To apply an equation to a *subterm* — rewriting `0 + 0` to `0` inside a larger
goal rather than matching the goal whole — you use the explicit congruence rules
`rewrite_r` / `rewrite_l` (see the next section). There is no hidden computation
step: every use of an equation is a rule you can point to in the proof.

The `core` module supplies `refl`, which proves any term equal to itself:

```alg
axiom refl(T : Sort, x : T)
  ⊢ x = x;
```

It takes the sort `T` and the term `x`, instantiated at the point of use — e.g.
`by refl(Nat, 0)` proves `0 = 0`. Because `defeq` is α/β only, `refl` closes a
goal `a = b` exactly when `a` and `b` are already α/β-equal; `refl(Nat, 0)` proves
`0 = 0`, but **not** `0 + 0 = 0`.

## Rules: proofs with subgoals

An **inference rule** has premises above a line and a conclusion below it. Applying a
rule to a goal that matches its conclusion produces one new subgoal per premise. Here
is `symmetry` from `core`:

```alg
rule symmetry(T : Sort, x y : T)
  ⊢ x = y
  ────────────────────────
  ⊢ y = x
end;
```

To use it, you `by symmetry(...)`. Its single premise leaves **one** remaining goal,
so you continue the same block with `then`: restate that goal and prove it with the
next `by`. Flipping `add_zero_left` — proving `n = 0 + n` from `0 + n = n`:

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by symmetry(Nat, 0 + n, n)   # conclusion y = x matches goal n = 0 + n
  then ⊢ 0 + n = n;            # the one remaining subgoal
  by add_zero_left(n);         # discharged by the axiom
qed;
```

The shape is always the same: **a step leaves subgoals; `then` continues a single
one, `cases` splits several.** An axiom (or any premise-free fact) generates *zero*
subgoals, so it closes a goal outright — that is why `by add_zero_left(n);` ends the
chain with no `then`.

### The arguments are inputs; the goal is matched

`by symmetry(T, a, b)` passes three *arguments* — matched against the rule's
parameters `(T, x, y)` and typechecked exactly like operator arguments. The current
*goal* is not passed; it is unified with the rule's conclusion. A rule adds only one
thing over an axiom: its premises become new subgoals.

Rule parameters can be terms, sorts, predicates (`P : T → Prop`), or **proof
arguments** — a parameter written `eq := a = b` expects a *proof reference* whose
statement is `a = b`, not a term. `rewrite_r` uses one:

```alg
rule rewrite_r(T : Sort, a b : T, eq := a = b, P : T → Prop)
  ⊢ P(a)
  ────────────────────────
  ⊢ P(b)
end;
```

## Proof blocks

A proof block is the keyword `proof`, a chain of `by` steps, and a terminator `qed`
(complete) or `wip` (in progress). Each `by` step has exactly one of three outcomes,
and its shape follows the number of subgoals it leaves:

- **zero** — `by refl(Nat, 0);` closes the goal.
- **one** — `by symmetry(...) then <goal>; by …` continues the *same* block, with no
  nesting; the `then` restates the single remaining subgoal.
- **many** — `by induction(...) cases <case> <case> …` branches, one `case` per
  subgoal (each `case` has its own nested `proof … qed`).

So a proof reads top to bottom: a straight `by … then … by …` chain for single-goal
steps, splitting into `cases` only where a rule genuinely branches. `then` may only
follow a step that leaves one goal, `cases` a step that leaves two or more, and a
`case` is legal only inside a `cases` block.

`by wip;` **admits** the current goal without proving it; a block that (transitively)
admits must be closed with `wip` instead of `qed`. `by wip(?name)` is the same admit
but also **reports a hole** at that goal — see
[Building a proof with holes](#building-a-proof-with-holes).

## Worked example: proof by induction

Now the real thing. `nat` defines addition and an induction rule whose conclusion is
a universally quantified proposition:

```alg
rule induction(P : Nat → Prop)
  ⊢ P(0);
  n : Nat, ih := P(n) ⊢ P(s(n))
  ──────────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;
```

Two premises — the base case `P(0)` and the step case (assuming `ih := P(n)`, prove
`P(s(n))`) — and a conclusion `∀ n. P(n)`. To prove `n + 0 = n` for all `n` we
apply it. The full proof from `nat.alg`:

```alg
import core(refl, rewrite_r, transitivity);

sort Nat : Sort;
op 0 : → Nat;
op s : Nat → Nat;
op + : Nat × Nat → Nat;

axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);

rule induction(P : Nat → Prop)
  ⊢ P(0);
  n : Nat, ih := P(n) ⊢ P(s(n))
  ──────────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;

lemma add_zero_right
  ⊢ ∀ (n : Nat) st n + 0 = n;
proof
  by induction(_ + 0 = _) cases       # motive P = (λ k. k + 0 = k)
    case
      ⊢ 0 + 0 = 0;                    # base: P(0)
    proof
      by add_zero_left(0);             # conclusion 0 + n = n at n = 0 is the goal
    qed;

    case
      k : Nat;
      ih := k + 0 = k;                 # step: assume P(k)
      ⊢ s(k) + 0 = s(k);             # prove P(s k)
    proof
      by rewrite_r(Nat, k + 0, k, ih, s(k) + 0 = s(_))
      then ⊢ s(k) + 0 = s(k + 0);      # goal after rewriting k <- k + 0
      by add_succ_left(k, 0);          # conclusion s(k) + 0 = s(k + 0) is the goal
    qed;
  qed;
qed;
```

Reading it as a tree:

- `by induction(_ + 0 = _)` supplies the **motive** `P` (the hole sugar `_ + 0 = _`
  desugars to `λ k. k + 0 = k`). The goal `∀ n. n + 0 = n` matches
  `induction`'s conclusion, producing **two** subgoals — one per `case`.
- The **base case** `0 + 0 = 0` is closed by `add_zero_left(0)`: its conclusion
  `0 + n = n`, instantiated at `n = 0`, is exactly `0 + 0 = 0`.
- The **step case** assumes `ih := k + 0 = k` and must prove `s(k) + 0 = s(k)`. It
  uses `ih` to rewrite `k` to `k + 0` under `s`, leaving `s(k) + 0 = s(k + 0)`, which
  `add_succ_left(k, 0)` discharges (its conclusion `s(k) + 0 = s(k + 0)` is that goal).

Notice `ih` — a **hypothesis** introduced by the step case — is used as a proof
argument to `rewrite_r`. That is the proof namespace at work: `ih` is not a term, it
is evidence.

### Eigenvariables

In the step case, `k` is an **eigenvariable**: a fresh variable standing for an
arbitrary `Nat`. Introducing it is the formal version of "let k be arbitrary." The
kernel enforces that such a variable is genuinely fresh (it may not already occur in
the surrounding context) — that freshness is what makes "prove `P(k)` for arbitrary
`k`" sound as "prove `∀ k. P(k)`".

## The two namespaces

Now the promised concreteness. Propositions are elaborated in the **term namespace**;
`by` references and proof arguments resolve in the **proof namespace**. The two never
mix.

The practical consequence: **a proposition cannot mention a proof-former.** Suppose
you declare an axiom `bar` and then try to use `bar(x)` as if it were a proposition:

```alg
# THIS DOES NOT COMPILE
import nat;

lemma oops
  ⊢ add_zero_left(0);   # error: `add_zero_left` is an axiom, not a term
proof
  by add_zero_left(0);
qed;
```

`verify` rejects it with `error: unbound name \`add_zero_left\``: the proposition
`add_zero_left(0)` is elaborated in the term namespace, and there is no *term* named
`add_zero_left` — only a proof-namespace axiom. To speak of a fact inside a
proposition you would have to declare an operator, e.g. `op even : Nat → Prop`. The
reverse is blocked too: an operator cannot be applied as a tactic in a `by`.

This is why axioms, rules, and lemmas are not "first-class values": you can reference
them in proofs, apply them, and pass hypotheses as evidence, but you cannot put them
in a proposition or quantify over them.

## Parameters vs. `forall`

A lemma can bind a variable two ways, and they read as the same theorem but behave
differently in proofs:

```alg
lemma foo(x : T) ⊢ P(x);            # a parameter
lemma foo         ⊢ ∀ (x : T) st P(x);   # a quantifier in the proposition
```

Both assert "P holds for every x." The difference is representation:

- A **parameter** `x` is a *schematic* variable — an implicit universal. As a proof
  step, `by foo(a)` instantiates it directly: it proves `P(a)` for any term `a`.
- A **`forall`** puts the universal *inside* the proposition, as an object-level
  connective you introduce and eliminate with explicit rules.

`core` provides the two bridges between them:

```alg
rule forall_intro(T : Sort, P : T → Prop)      rule forall_elim(T : Sort, P : T → Prop)
  x : T ⊢ P(x)                                   ⊢ ∀ (y : T) st P(y)
  ────────────────────────                        ────────────────────────
  ⊢ ∀ (x : T) st P(x)                       x : T ⊢ P(x)
```

`forall_intro` turns a proof of `P(x)` for a fresh eigenvariable `x` into
`∀ x. P(x)`; `forall_elim` goes the other way. Proving a `forall` goal therefore
starts by introducing the variable:

```alg
import nat;
import core(refl, forall_intro);

lemma all_refl
  ⊢ ∀ (n : Nat) st n = n;
proof
  by forall_intro(Nat, λ (n : Nat) st n = n)
  then n : Nat ⊢ n = n;    # n introduced as an eigenvariable
  by refl(Nat, n);
qed;
```

Here the `then` keeps its context: `forall_intro` introduces the fresh eigenvariable
`n`, so the continuation names it (`n : Nat ⊢ …`). When a step introduces no new
variables — most `rewrite_r` steps — you can drop the context and write just
`then ⊢ <goal>;`.

This is also why `induction` states its conclusion as `∀ (n : Nat) st P(n)`
rather than taking `n` as a parameter: the step case must reason about `n` as a bound
eigenvariable that the premises discharge, which a caller-supplied parameter could
not express.

## Holes

Writing motives by hand is verbose, so `_` is sugar for a lambda. In the induction
proof, `by induction(_ + 0 = _)` means `by induction(λ k. k + 0 = k)`: each `_`
becomes the lambda's bound variable. Use holes wherever a predicate argument is
"obvious from the goal."

## Building a proof with holes

You don't have to write a proof top to bottom in one shot. `by wip(?name)` **admits**
the current goal like `by wip`, but also prints a **hole report** — the goal, the
context in scope, and candidate tactics — so you can grow a proof one step at a time.
The report shows on the command line *and* inline in the editors below (press
**Check ▶**).

Start with just the skeleton and a hole:

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by wip(?goal);
wip;
```

Checking it reports the goal, what is in scope, and where to look next:

```text
found hole ?goal : proof

Expected:
  n = 0 + n

Context:
  n : Nat

Goal:
  n = 0 + n

Candidates:
  symmetry (rule)
  transitivity (rule)
```

`symmetry` rewrites `n = 0 + n` into `0 + n = n`. Apply it, and move the hole into the
`then` continuation to see what is left:

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by symmetry(Nat, 0 + n, n)
  then ⊢ 0 + n = n;
  by wip(?rest);
wip;
```

Now the hole reports `0 + n = n`, with `add_zero_left (fact)` among the candidates —
exactly the axiom that closes it. Drop it in and swap the final `wip` for `qed`:

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by symmetry(Nat, 0 + n, n)
  then ⊢ 0 + n = n;
  by add_zero_left(n);
qed;
```

A module with any `wip` — holed or not — is **incomplete**: the checker reports it and
the run fails, so a hole can never masquerade as a finished proof. Candidates are a
best-effort hint (local hypotheses, facts and rules whose conclusion matches the goal
shape, and `refl` for a reflexive equation), not a guarantee — but they are usually
enough to find the next `by`.

### Holes inside a tactic

Once you have picked a tactic, a `?` helps you *fill it in*. Put `?` after a whole
application to **inspect** it — the checker applies the tactic and hands you the next
step, ready to paste:

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by symmetry(Nat, 0 + n, n)?;
wip;
```

```text
Applying it leaves:
  ⊢ 0 + n = n

Continue with:
  then ⊢ 0 + n = n;
  by wip?;
```

Or leave individual arguments as **named holes** `?a` and let the checker solve them
from the goal. `symmetry`'s conclusion `y = x` must match `n = 0 + n`, which forces
`?a` and `?b` (and even the sort `?T`, recovered by type inference):

```alg
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by symmetry(Nat, ?a, ?b) then ?g;
wip;
```

```text
Holes:
  ?a : Nat = 0 + n
  ?b : Nat = n

Subgoal(s):
  ?g : ⊢ 0 + n = n
```

`by symmetry?;` (no arguments) holes *every* parameter at once. Holes also work in
proof-argument positions: `by rewrite_r(Nat, k + 0, k, ?eq, _)?;` reports
`?eq : ⊢ k + 0 = k` — the equation you still owe a proof of. A hole an argument does
not pin down (a genuinely free choice, like `transitivity`'s middle term) is shown
with its type and no value.

## Theories, laws, and models

Beyond single facts, Algae groups requirements into **theories** and discharges them
with **models**. A **theory** is a parameterized interface plus a list of **laws**
(required propositions). `group.alg` builds the classic algebra hierarchy, each
theory `include`-ing the previous one and adding laws:

```alg
theory Monoid(
  S : Sort,
  mul : S × S → S,
  e : S
) laws
  include Semigroup(S, mul);            # associativity, inherited

  law left_identity(x : S)   ⊢ mul(e, x) = x;
  law right_identity(x : S)  ⊢ mul(x, e) = x;
qed;
```

A **model** claims that specific operators satisfy a theory, and must prove each law
as an obligation. Here is the `Monad` interface from `monad.alg`:

```alg
theory Monad(
  A B C : Sort,
  M : Sort → Sort,
  return : A → M(A),
  bind : M(A) × (A → M(B)) → M(B)
) laws
  law left_identity(x : A, f : A → M(B))  ⊢ bind(return(x), f) = f(x);
  law right_identity(m : M(A))            ⊢ bind(m, return) = m;
  law associativity(m : M(A), f : A → M(B), g : B → M(C))
    ⊢ bind(bind(m, f), g) = bind(m, λ (x : A) st bind(f(x), g));
qed;
```

`option.alg` declares `Option`, `return`, `bind`, then proves they form a monad. The
first law is `bind(return(x), f) = f(x)`. Since `return(x)` equals `some(x)` only
*through* the axiom `return_def` — not by computation — the proof **rewrites**
`return(x)` to `some(x)` with `rewrite_r`, then applies `bind_some`:

```alg
model OptionMonad satisfies Monad(
  A, B, C, Option, return, bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      Option(A),
      return(x), some(x),
      return_def(A, x),                       # return(x) = some(x)
      λ (o : Option(A)) st bind(o, f) = f(x)
    )
    case
      A B : Sort;
      x : A;
      f : A → Option(B);
      ⊢ bind(some(x), f) = f(x);
    proof
      by bind_some(A, B, x, f);
    qed;
  qed;

  # ... right_identity and associativity follow
qed;
```

Each `law <name>;` in the model names one obligation from the theory and proves it
just like a lemma. This is also the discipline from *[Axioms and definitional
equality](#axioms-and-definitional-equality)* at scale: every one of the monad-law
proofs in `option.alg`, `list.alg`, and `result.alg` reaches its equalities through
explicit `rewrite_r` / `rewrite_l` steps, never by silent evaluation.

## Imports and the standard library

`import module;` brings **everything** a module declares into scope — its sorts,
operators, axioms, and rules. `import module(name, …)` selects specific names, and
`import module(name as alias)` renames. Either way the module's operators become
usable (that is why `import nat;` above let us write `0` and `+`).

The standard library lives in `algae/stdlib/v1/`:

| module | what it provides |
|--------|------------------|
| `core` | equality (`refl`, `symmetry`, `rewrite_r`/`rewrite_l`), logic, quantifiers |
| `nat` | `Nat`, `+`, `×`, and `induction` |
| `option`, `result`, `list` | data types with their `Monad` models |
| `monad` | the `Functor` / `Applicative` / `Monad` theories |
| `group` | the `Magma` → … → `AbelianGroup` theory hierarchy |
| `adt` | algebraic-datatype scaffolding |

Verify the whole library with:

```sh
cargo run -p algae-cli -- verify algae/stdlib/v1/
```

## Where to go next

- `algae/stdlib/v1/` — worked, verified modules to read and imitate.
- `lang-specs/spec.md` (in the repository) — the precise grammar and static semantics.
- `tests/accept/` — one minimal proof per inference rule.
