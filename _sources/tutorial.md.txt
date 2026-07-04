# Algae Tutorial

Algae is a small proof and algebraic-specification language. You declare a
vocabulary (sorts, operators), assert facts (axioms) and inference rules, and then
prove lemmas by writing **explicit proof trees** that a tiny trusted kernel
re-checks. This tutorial builds up from a one-line proof to a proof by induction.

The companion [`spec.md`](spec.md) is the precise reference; this document is the
gentle path in.

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
This tutorial uses ASCII so it is easy to type; `fmt` converts to Unicode.

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
signature. Nothing here is a proof yet — this is the term world's vocabulary.

```alg
sort Nat : Sort;              # a base sort

op 0 : -> Nat;               # a nullary operator (a constant)
op s : Nat -> Nat;           # successor
op + : Nat * Nat -> Nat;     # a binary operator, written infix as x + y
```

Types are built from sorts with `*` (product), `|` (sum), and `->` (function). A
proposition has the special type `Prop`; a predicate is therefore an operator into
`Prop`, e.g. `op even : Nat -> Prop`.

## Propositions and sequents

A **proposition** is just a `Prop`-valued term: an equation `a = b`, a connective
(`/\`, `\/`, `=>`, `<=>`, `~`), a quantifier (`forall`, `exists`), or a predicate
applied to arguments. Terms and propositions share one grammar.

A **sequent** is a proposition under a context of assumptions:

```
context |- proposition
```

The context lists typed variables and named hypotheses. With an empty context you
just write `|- proposition`. These two are read "under x and y, and a proof of
x = y, conclude y = x":

```alg
|- x = x
x : Nat, y : Nat, h := x = y |- y = x
```

A **lemma** states a sequent and must supply a proof of it.

## Axioms and definitional equality

An **axiom** asserts a sequent as true without proof. Operators get their meaning
from equational axioms:

```alg
axiom add_zero_left(n : Nat)
  |- 0 + n = n;
```

Crucially, the checker treats equational axioms as **rewrite rules** and compares
terms *up to computation* — this is **definitional equality** (`defeq`). Two terms
are equal if they reduce to the same normal form. So once `0 + n = n` is an axiom,
`0 + 0` and `0` are interchangeable, and this proof goes through:

```alg
import core(refl);

sort Nat : Sort;
op 0 : -> Nat;
op + : Nat * Nat -> Nat;
axiom add_zero_left(n : Nat)
  |- 0 + n = n;

lemma zero_plus_zero
  |- 0 + 0 = 0;
proof
  by refl(Nat, 0);
qed;
```

`refl(Nat, 0)` proves `0 = 0`. The goal is `0 + 0 = 0`, but `0 + 0` normalizes to
`0`, so the goal *is* `0 = 0` up to `defeq`, and the proof is accepted. Computation
happens for free through definitional equality; there is no separate "compute" step.

`refl` is the first thing in the standard `core` module:

```alg
axiom refl(T : Sort, x : T)
  |- x = x;
```

Note it takes the sort `T` and the term `x` as arguments — you instantiate it at the
point of use, e.g. `by refl(Nat, 0)`.

## Rules: proofs with subgoals

An **inference rule** has premises above a line and a conclusion below it. Applying a
rule to a goal that matches its conclusion produces one new subgoal per premise. Here
is `symmetry` from `core`:

```alg
rule symmetry(T : Sort, x y : T)
  |- x = y
  ------------------------
  |- y = x
end;
```

To use it, you `by symmetry(...)` and then supply a `case` for its single premise:

```alg
import core(symmetry);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma b_eq_a
  |- b = a;
proof
  by symmetry(T, a, b)     # conclusion y = x matches goal b = a
  case
    |- a = b;              # the premise becomes this subgoal
  proof
    by ab;                # discharged by the axiom
  qed;
qed;
```

The shape is always the same: **a rule application generates subgoals; each `case`
proves one.** An axiom (or any premise-free fact) generates *zero* subgoals, so it
closes a goal outright — that is why `by ab;` needs no `case`.

### The arguments are inputs; the goal is matched

`by symmetry(T, a, b)` passes three *arguments* — matched against the rule's
parameters `(T, x, y)` and typechecked exactly like operator arguments. The current
*goal* is not passed; it is unified with the rule's conclusion. A rule adds only one
thing over an axiom: its premises become new subgoals.

Rule parameters can be terms, sorts, predicates (`P : T -> Prop`), or **proof
arguments** — a parameter written `eq := a = b` expects a *proof reference* whose
statement is `a = b`, not a term. `rewrite_r` uses one:

```alg
rule rewrite_r(T : Sort, a b : T, eq := a = b, P : T -> Prop)
  |- P(a)
  ------------------------
  |- P(b)
end;
```

## Proof blocks

A proof block is the keyword `proof`, **exactly one** `by` statement, and a
terminator `qed` (complete) or `wip` (in progress):

```
proof by <ref> <cases?> <terminator> ;
```

A `by` statement comes in three shapes, by how many subgoals the rule produces:

- **zero** — `by refl(Nat, 0);` closes the goal.
- **one** — `by symmetry(...) case … qed;` (a single `case`).
- **many** — `by induction(...) cases <case> <case> … qed;` (the `cases` keyword,
  one `case` per premise).

There is no way to write two `by` statements in one block — the parser rejects it.
Proofs grow *downward* through the sub-proofs of `case`s, not sideways.

`by wip;` **admits** the current goal without proving it. A block that (transitively)
contains an admit must be closed with `wip` instead of `qed`; the checker skips
admitted goals but the CLI reports the proof as incomplete and fails the run.

## Worked example: proof by induction

Now the real thing. `nat` defines addition and an induction rule whose conclusion is
a universally quantified proposition:

```alg
rule induction(P : Nat -> Prop)
  |- P(0);
  n : Nat, ih := P(n) |- P(s(n))
  ------------------------------
  |- forall (n : Nat) st P(n)
end;
```

Two premises — the base case `P(0)` and the step case (assuming `ih := P(n)`, prove
`P(s(n))`) — and a conclusion `forall n. P(n)`. To prove `n + 0 = n` for all `n` we
apply it. The full proof from `nat.alg`:

```alg
import core(refl, rewrite_r, transitivity);

sort Nat : Sort;
op 0 : -> Nat;
op s : Nat -> Nat;
op + : Nat * Nat -> Nat;

axiom add_zero_left(n : Nat)     |- 0 + n = n;
axiom add_succ_left(n m : Nat)   |- s(n) + m = s(n + m);

rule induction(P : Nat -> Prop)
  |- P(0);
  n : Nat, ih := P(n) |- P(s(n))
  ------------------------------
  |- forall (n : Nat) st P(n)
end;

lemma add_zero_right
  |- forall (n : Nat) st n + 0 = n;
proof
  by induction(_ + 0 = _) cases       # motive P = (lambda k. k + 0 = k)
    case
      |- 0 + 0 = 0;                    # base: P(0)
    proof
      by add_zero_left(0);             # 0 + 0 reduces to 0
    qed;

    case
      k : Nat;
      ih := k + 0 = k;                 # step: assume P(k)
      |- s(k) + 0 = s(k);             # prove P(s k)
    proof
      by rewrite_r(Nat, k + 0, k, ih, s(k) + 0 = s(_))
      case
        k : Nat;
        ih := k + 0 = k;
        |- s(k) + 0 = s(k + 0);        # goal after rewriting k <- k+0
      proof
        by add_succ_left(k, 0);        # s(k) + 0 reduces to s(k + 0)
      qed;
    qed;
  qed;
qed;
```

Reading it as a tree:

- `by induction(_ + 0 = _)` supplies the **motive** `P` (the hole sugar `_ + 0 = _`
  desugars to `lambda k. k + 0 = k`). The goal `forall n. n + 0 = n` matches
  `induction`'s conclusion, producing **two** subgoals — one per `case`.
- The **base case** `0 + 0 = 0` is closed by `add_zero_left(0)`: `0 + 0` reduces to
  `0`, so the goal is `0 = 0` up to `defeq`.
- The **step case** assumes `ih := k + 0 = k` and must prove `s(k) + 0 = s(k)`. It
  uses `ih` to rewrite `k` to `k + 0` under `s`, leaving `s(k) + 0 = s(k + 0)`, which
  `add_succ_left(k, 0)` discharges (`s(k) + 0` reduces to `s(k + 0)`).

Notice `ih` — a **hypothesis** introduced by the step case — is used as a proof
argument to `rewrite_r`. That is the proof namespace at work: `ih` is not a term, it
is evidence.

### Eigenvariables

In the step case, `k` is an **eigenvariable**: a fresh variable standing for an
arbitrary `Nat`. Introducing it is the formal version of "let k be arbitrary." The
kernel enforces that such a variable is genuinely fresh (it may not already occur in
the surrounding context) — that freshness is what makes "prove `P(k)` for arbitrary
`k`" sound as "prove `forall k. P(k)`".

## The two namespaces

Now the promised concreteness. Propositions are elaborated in the **term namespace**;
`by` references and proof arguments resolve in the **proof namespace**. The two never
mix.

The practical consequence: **a proposition cannot mention a proof-former.** Suppose
you declare an axiom `bar` and then try to use `bar(x)` as if it were a proposition:

```alg
# THIS DOES NOT COMPILE
import core(refl);
sort T : Sort;
op a : -> T;
axiom bar(x : T) |- x = x;

lemma oops
  |- bar(a);          # error: `bar` is a proof-former, not a term
proof
  by refl(T, a);
qed;
```

`verify` rejects it with `error: unbound name \`bar\``: `bar(a)` is elaborated in the
term namespace, and there is no *term* named `bar` — only a proof-namespace axiom. To
speak of `bar` inside a proposition you would have to declare it as an operator,
`op bar : T -> Prop`. The reverse is blocked too: an operator cannot be applied as a
tactic in a `by`.

This is why axioms, rules, and lemmas are not "first-class values": you can reference
them in proofs, apply them, and pass hypotheses as evidence, but you cannot put them
in a proposition or quantify over them.

## Parameters vs. `forall`

A lemma can bind a variable two ways, and they read as the same theorem but behave
differently in proofs:

```alg
lemma foo(x : T) |- P(x);            # a parameter
lemma foo         |- forall (x : T) st P(x);   # a quantifier in the proposition
```

Both assert "P holds for every x." The difference is representation:

- A **parameter** `x` is a *schematic* variable — an implicit universal. As a proof
  step, `by foo(a)` instantiates it directly: it proves `P(a)` for any term `a`.
- A **`forall`** puts the universal *inside* the proposition, as an object-level
  connective you introduce and eliminate with explicit rules.

`core` provides the two bridges between them:

```alg
rule forall_intro(T : Sort, P : T -> Prop)      rule forall_elim(T : Sort, P : T -> Prop)
  x : T |- P(x)                                   |- forall (y : T) st P(y)
  ------------------------                        ------------------------
  |- forall (x : T) st P(x)                       x : T |- P(x)
```

`forall_intro` turns a proof of `P(x)` for a fresh eigenvariable `x` into
`forall x. P(x)`; `forall_elim` goes the other way. Proving a `forall` goal therefore
starts by introducing the variable:

```alg
import core(refl, forall_intro);

sort T : Sort;

lemma all_refl
  |- forall (x : T) st x = x;
proof
  by forall_intro(T, lambda (x : T) st x = x)
  case
    x : T |- x = x;         # x introduced as an eigenvariable
  proof
    by refl(T, x);
  qed;
qed;
```

This is also why `induction` states its conclusion as `forall (n : Nat) st P(n)`
rather than taking `n` as a parameter: the step case must reason about `n` as a bound
eigenvariable that the premises discharge, which a caller-supplied parameter could
not express.

## Holes

Writing motives by hand is verbose, so `_` is sugar for a lambda. In the induction
proof, `by induction(_ + 0 = _)` means `by induction(lambda k. k + 0 = k)`: each `_`
becomes the lambda's bound variable. Use holes wherever a predicate argument is
"obvious from the goal."

## Theories, laws, and models (brief)

Beyond single facts, Algae groups requirements into **theories** and discharges them
with **models**. A theory lists **laws** (required propositions); a model claims to
satisfy a theory and must prove each law as an obligation. See `group.alg` and
`monad.alg` in the standard library, and §3.9–3.12 / §4.10–4.13 of the spec, for the
full story.

## Imports and the standard library

`import module(name, …)` brings names into scope; `import module(name as alias)`
renames. The standard library lives in `algae/stdlib/v1/` — `core` (equality, logic,
quantifiers), `nat`, `list`, `option`, `result`, `monad`, `adt`, `group`. Verify it
all with:

```sh
cargo run -p algae-cli -- verify algae/stdlib/v1/
```

## Where to go next

- [`spec.md`](spec.md) — the precise grammar and static semantics.
- `algae/stdlib/v1/` — worked, verified modules to read and imitate.
- `tests/accept/` — one minimal proof per inference rule.
