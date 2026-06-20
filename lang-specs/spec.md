# Algae v2 Language Specification

## 1. Purpose

Algae v2 is a parser-oriented proof and algebraic specification language.

The language supports:

* typed sort constructors;
* total operators;
* product types;
* sum types;
* propositions;
* sequents;
* axioms;
* inference rules;
* explicit proof trees;
* theories;
* laws;
* models;
* imports;
* qualified names.

Algae v2 does not support partial functions. All functions are total. Failure is modeled using sum types.

---

# 2. Lexical Structure

## 2.1 Comments

A comment starts with `#` and continues to the end of the line.

```alg
# This is a comment.
sort Nat : Sort;
```

## 2.2 Whitespace

Whitespace separates tokens but is otherwise insignificant.

## 2.3 Identifiers

An identifier starts with a letter or `_`, followed by letters, digits, or `_`.

Examples:

```text
Nat
Option
add_zero_left
OptionMonad
```

## 2.4 Qualified Identifiers

A qualified identifier is:

```text
module.symbol
```

Examples:

```alg
core.reflexivity
nat.Nat
monad.Monad
```

## 2.5 Reserved Keywords

```text
import
sort
op
axiom
rule
lemma
theorem
proof
qed
wip
case
cases
props
laws
theory
law
model
satisfies
iff
include
forall
exists
st
as
end
Sort
Prop
False
```

## 2.6 Preferred ASCII and Unicode Alternatives

Both forms are accepted.

| Meaning           |                      ASCII |                    Unicode |
| ----------------- | -------------------------: | -------------------------: |
| sequent turnstile |                      `\|-` |                        `⊢` |
| product type      |                        `*` |                        `×` |
| sum type          |                       `\|` |                            |
| lambda            |                   `lambda` |                        `λ` |
| universal         |                   `forall` |                        `∀` |
| existential       |                   `exists` |                        `∃` |
| conjunction       |                       `/\` |                        `∧` |
| disjunction       |                       `\/` |                        `∨` |
| implication       |                       `=>` |                        `⇒` |
| arrow             |                       `->` |                        `→` |
| biconditional     |                      `<=>` |                        `⇔` |
| negation          |                        `~` |                        `¬` |
| separator         | `------------------------` | `────────────────────────` |

The symbol `->` is only for function types.

The symbol `=>` is only for logical implication.

---

# 3. Grammar

## 3.1 File

```ebnf
file =
  { top_decl } ;
```

## 3.2 Top-Level Declarations

```ebnf
top_decl =
    import_decl
  | sort_decl
  | op_decl
  | axiom_decl
  | rule_decl
  | lemma_decl
  | theorem_decl
  | theory_decl
  | model_decl ;
```

---

## 3.3 Imports

```ebnf
import_decl =
  "import" module_name [ import_list ] ";" ;

module_name =
  ident ;

import_list =
  "(" [ import_item_list ] ")" ;

import_item_list =
  import_item { "," import_item } ;

import_item =
    ident
  | ident "as" ident ;
```

Examples:

```alg
import core;

import nat(
  Nat,
  s
);

import monad(
  Monad as MonadTheory
);
```

---

## 3.4 Sort Declarations

```ebnf
sort_decl =
  "sort" sort_binding_list ";" ;

sort_binding_list =
  sort_binding { "," sort_binding } ;

sort_binding =
  ident_list ":" kind_expr ;

ident_list =
  ident { ident } ;
```

Examples:

```alg
sort Nat : Sort;

sort A B Err : Sort;

sort Option : Sort -> Sort;

sort Result : Sort * Sort -> Sort;
```

---

## 3.5 Operator Declarations

```ebnf
op_decl =
  "op" symbol ":" function_sig ";" ;

function_sig =
  [ type_expr ] "->" type_expr ;
```

Examples:

```alg
op 0 : -> Nat;

op s : Nat -> Nat;

op + : Nat * Nat -> Nat;

op may_fail : A -> B | Err;
```

---

## 3.6 Axioms

```ebnf
axiom_decl =
  "axiom" ident formal_params sequent ";" ;
```

Example:

```alg
axiom reflexivity(
  T : Sort,
  x : T
)
  |- x = x;
```

---

## 3.7 Rules

```ebnf
rule_decl =
  "rule" ident formal_params rule_body "end" ";" ;

rule_body =
  premise_list separator sequent ;

premise_list =
  sequent { ";" sequent } ;

separator =
    "------------------------"
  | "────────────────────────" ;
```

Example:

```alg
rule transitivity(
  T : Sort,
  x y z : T
)
  |- x = y;
  |- y = z
  ------------------------
  |- x = z
end;
```

---

## 3.8 Lemmas and Theorems

```ebnf
lemma_decl =
  "lemma" ident formal_params sequent ";" proof_block ;

theorem_decl =
  "theorem" ident formal_params sequent ";" proof_block ;
```

Example:

```alg
lemma add_zero_right(
  n : Nat
)
  |- n + 0 = n;
proof
  by reflexivity(Nat, n);
qed;
```

---

## 3.9 Theories

```ebnf
theory_decl =
  "theory" ident formal_params "laws" theory_item_list "qed" ";" ;

theory_item_list =
  { theory_item } ;

theory_item =
    include_decl
  | law_decl ;
```

Example:

```alg
theory Semigroup(
  S : Sort,
  * : S * S -> S
) laws
  law associativity(
    x y z : S
  )
    |- *( *(x, y), z ) = *( x, *(y, z) );
qed;
```

---

## 3.10 Theory Includes

```ebnf
include_decl =
  "include" ident actual_args ";" ;
```

Example:

```alg
include Semigroup(S, *);
```

---

## 3.11 Laws

Laws are syntactically compatible with axioms.

```ebnf
law_decl =
  "law" ident formal_params sequent ";" ;
```

Example:

```alg
law left_identity(
  x : S
)
  |- *(e, x) = x;
```

---

## 3.12 Models

```ebnf
model_decl =
  "model" ident "satisfies" ident actual_args "iff"
  "props"
    model_law_list
  terminator
  ";" ;

model_law_list =
  { model_law } ;

model_law =
  "law" qualified_or_unqualified_ident ";" proof_block ;
```

Example:

```alg
model NatAddMonoid satisfies Monoid(
  Nat,
  0,
  +
) iff props
  law Semigroup.associativity;
  proof
    by add_associativity;
  qed;

  law left_identity;
  proof
    by add_zero_left;
  qed;

  law right_identity;
  proof
    by add_zero_right;
  qed;
qed;
```

---

## 3.13 Proof Blocks

```ebnf
proof_block =
  "proof" proof_stmt_list terminator ";" ;

terminator =
    "qed"            (* a complete, sound proof *)
  | "wip" ;          (* an in-progress proof (contains an admit) *)

proof_stmt_list =
  { proof_stmt } ;

proof_stmt =
    by_stmt_wip ";"
  | by_stmt_zero ";"
  | by_stmt_one
  | by_stmt_many ";" ;

by_stmt_wip =
  "by" "wip" ;        (* admit the current goal without a proof *)

by_stmt_zero =
  "by" proof_ref ;

by_stmt_one =
  "by" proof_ref case_block ;

by_stmt_many =
  "by" proof_ref "cases" case_block { case_block } terminator ;
```

---

## 3.14 Proof References

```ebnf
proof_ref =
    qualified_or_unqualified_ident
  | qualified_or_unqualified_ident actual_args ;

qualified_or_unqualified_ident =
    ident
  | ident "." ident ;

actual_args =
  "(" [ term_list ] ")" ;

term_list =
  term { "," term } ;
```

---

## 3.15 Cases

```ebnf
case_block =
  "case" case_body proof_block ;

case_body =
  [ context ] sequent_goal ;

sequent_goal =
  ("|-" | "⊢") prop ";" ;
```

Example:

```alg
case
  n : Nat;
  ih := P(n);
  |- P(s(n));
proof
  by step_case(n, ih);
qed;
```

---

## 3.16 Sequents

```ebnf
sequent =
  [ context ] ("|-" | "⊢") prop ;
```

Examples:

```alg
|- x = x
```

```alg
x : Nat, y : Nat |- x = y => y = x
```

---

## 3.17 Contexts

```ebnf
context =
  context_entry { context_sep context_entry } context_sep? ;

context_sep =
    ","
  | ";" ;

context_entry =
    term_binding
  | proof_binding ;

term_binding =
  ident_list ":" type_expr ;

proof_binding =
  ident ":=" prop ;
```

Examples:

```alg
x : Nat
```

```alg
x y z : Nat
```

```alg
ih := P(n)
```

---

## 3.18 Formal Parameters

```ebnf
formal_params =
  "(" [ formal_param_list ] ")" ;

formal_param_list =
  formal_param { "," formal_param } ;

formal_param =
    term_binding
  | proof_binding ;
```

Examples:

```alg
()
```

```alg
(n : Nat)
```

```alg
(x y z : Nat)
```

```alg
(eq := x = y)
```

---

## 3.19 Kinds

```ebnf
kind_expr =
  kind_function ;

kind_function =
  kind_product [ "->" kind_function ] ;

kind_product =
  kind_atom { ("*" | "×") kind_atom } ;

kind_atom =
    "Sort"
  | "(" kind_expr ")" ;
```

Examples:

```alg
Sort
```

```alg
Sort -> Sort
```

```alg
Sort * Sort -> Sort
```

---

## 3.20 Types

```ebnf
type_expr =
  function_type ;

function_type =
  sum_type [ "->" function_type ] ;

sum_type =
  product_type { "|" product_type } ;

product_type =
  type_atom { ("*" | "×") type_atom } ;

type_atom =
    qualified_or_unqualified_ident
  | type_application
  | "Prop"
  | "(" type_expr ")" ;

type_application =
  qualified_or_unqualified_ident "(" [ type_expr_list ] ")" ;

type_expr_list =
  type_expr { "," type_expr } ;
```

Examples:

```alg
Nat
```

```alg
Option(Nat)
```

```alg
Result(A, Err)
```

```alg
A * B
```

```alg
A | Err
```

```alg
A -> B
```

```alg
A -> B | Err
```

This parses as:

```alg
A -> (B | Err)
```

---

## 3.21 Propositions

Operator precedence from strongest to weakest:

1. `~` / `¬`
2. `=`
3. `/\` / `∧`
4. `\/` / `∨`
5. `=>` / `⇒`
6. `<=>` / `⇔`

```ebnf
prop =
  biconditional_prop ;

biconditional_prop =
  implication_prop
  { ("<=>" | "⇔") implication_prop } ;

implication_prop =
  disjunction_prop
  { ("=>" | "⇒") disjunction_prop } ;

disjunction_prop =
  conjunction_prop
  { ("\\/" | "∨") conjunction_prop } ;

conjunction_prop =
  negation_prop
  { ("/\\" | "∧") negation_prop } ;

negation_prop =
    ("~" | "¬") negation_prop
  | quantified_prop
  | equality_prop
  | prop_atom ;

quantified_prop =
    "forall" binder "st" prop
  | "exists" binder "st" prop ;

equality_prop =
  term "=" term ;

prop_atom =
    "False"
  | application
  | qualified_or_unqualified_ident
  | "(" prop ")" ;
```

---

## 3.22 Binders

```ebnf
binder =
  "(" term_binding ")" ;
```

Examples:

```alg
(x : Nat)
```

```alg
(A B : Sort)
```

---

## 3.23 Terms

```ebnf
term =
  lambda_term ;

lambda_term =
    ("lambda" | "λ") binder "st" term
  | infix_term ;

infix_term =
  application_term { infix_op application_term } ;

application_term =
  term_atom [ "(" [ term_list ] ")" ] ;

term_atom =
    qualified_or_unqualified_ident
  | numeric_symbol
  | symbolic_operator
  | hole
  | "(" term ")" ;

hole =
  "_" ;   (* sugar: an expression with holes is a unary lambda; see §4.16 *)

infix_op =
    "+"
  | "-"
  | "*"
  | "/"
  | qualified_or_unqualified_ident ;

symbol =
    qualified_or_unqualified_ident
  | numeric_symbol
  | symbolic_operator ;

numeric_symbol =
  digit { digit } ;

symbolic_operator =
  "+"
  | "-"
  | "*"
  | "/"
  | "=="
  | "<"
  | ">"
  | "<="
  | ">=" ;
```

---

# 4. Static Semantics

## 4.1 Namespaces

The language has separate namespaces for:

* modules;
* sorts;
* operators;
* axioms;
* rules;
* lemmas;
* theorems;
* theories;
* laws inside theories;
* models;
* local term variables;
* local proof variables.

## 4.2 Import Semantics

Given:

```alg
import foo(
  sym2,
  sym3 as alias3
);
```

the compiler loads `foo.alg`.

Every exported symbol from `foo.alg` is available as:

```alg
foo.symbol
```

Additionally:

```alg
sym2
```

is available as an alias for:

```alg
foo.sym2
```

and:

```alg
alias3
```

is available as an alias for:

```alg
foo.sym3
```

No other unqualified names are introduced.

## 4.3 Sort Semantics

A declaration:

```alg
sort Nat : Sort;
```

introduces a sort named `Nat`.

A declaration:

```alg
sort Option : Sort -> Sort;
```

introduces a sort constructor named `Option`.

If:

```alg
A : Sort
```

then:

```alg
Option(A) : Sort
```

## 4.4 Operator Semantics

An operator declaration introduces a total function.

Example:

```alg
op may_fail : A -> B | Err;
```

means:

* `may_fail` is total;
* for every value of sort `A`, it returns a value in the sum type `B | Err`.

## 4.5 Sum Type Semantics

A type:

```alg
A | B
```

is a disjoint sum type.

Values of `A` inhabit `A | B`.

Values of `B` inhabit `A | B`.

Proofs over sums use sum elimination rules.

## 4.6 Product Type Semantics

A type:

```alg
A * B
```

is a product type.

Values are pairs containing an `A` and a `B`.

Proofs over products use product elimination rules.

## 4.7 Axiom Semantics

An axiom is assumed true.

An axiom may be used as a proof reference.

## 4.8 Rule Semantics

A rule has premises and a conclusion.

Applying a rule to prove its conclusion generates proof obligations for its premises.

If a rule has zero premises, it closes the current goal.

If a rule has one premise, the proof uses one `case`.

If a rule has multiple premises, the proof uses `{ case ... case ... }`.

## 4.9 Lemma and Theorem Semantics

A lemma or theorem is valid only if its proof block proves its stated sequent.

After verification, it becomes available as a proof reference.

## 4.10 Law Semantics

A law is a required proposition inside a theory.

A law is not assumed true globally.

A law becomes a proof obligation when a model satisfies the theory.

## 4.11 Theory Semantics

A theory is a parameterized collection of laws.

A theory may include another theory.

The law set of a theory is:

```text
local laws union inherited laws from included theories
```

The inherited law set is transitive.

## 4.12 Include Semantics

Inside a theory:

```alg
include Semigroup(S, *);
```

means:

* the current theory depends on `Semigroup`;
* every model of the current theory must also satisfy `Semigroup(S, *)`;
* all laws of `Semigroup(S, *)` become obligations of the current theory.

This is not textual inclusion.

This is theory inheritance.

## 4.13 Model Semantics

A model declaration:

```alg
model M satisfies T(args) iff props
  law L1;
  proof
    ...
  qed;
qed;
```

is valid iff:

* `T(args)` is well-kinded and well-typed;
* every law in the transitive law set of `T` is proven exactly once;
* every provided law name belongs to the transitive law set of `T`;
* each law proof verifies against the instantiated law statement.

After verification, the environment records:

```text
M satisfies T(args)
```

## 4.14 Proof Binding Semantics

A context entry:

```alg
ih := P(n)
```

introduces a local proof variable named `ih`.

It may be used as a proof reference.

## 4.15 Generalization Rule Side Condition

The rule `generalization` may only be applied if the generalized variable is not free in any undischarged proof assumption. This is enforced as eigenvariable freshness: the variable is introduced in the rule's premise context, and a case's freshly introduced variable must not already occur in the surrounding context.

This is a proof-checking side condition, not a grammar condition.

## 4.16 Holes

An expression containing one or more `_` holes is sugar for a unary lambda. All
`_` in a single expression expand to the **same** fresh variable, and the
binder's type is inferred from the function domain of the tactic parameter the
expression is passed to. For example, where a tactic expects `P : Nat -> Prop`:

```alg
by induction(_ + 0 = _)
```

is sugar for:

```alg
by induction(lambda (k : Nat) st k + 0 = k)
```

A hole is only valid as a tactic argument whose parameter has a function type;
elsewhere it is a static error.

## 4.17 Work in progress (`wip`)

`by wip` admits the current goal without a proof (like an axiom assumption). A
proof that admits any goal must be closed by `wip` instead of `qed`; `qed` is
accepted only on complete, sound proofs. The `wip` terminator is **viral**:
every enclosing block (`proof`, `cases`, `props`) that transitively contains an
admit must also be closed by `wip` rather than `qed`.

The checker skips admitted goals and verifies the remaining (sound) parts, but a
unit containing any `wip` is reported as in progress and fails verification
overall.

---

# 5. Standard Module: core.alg

```alg
sort Bool : Sort;

op true : -> Bool;
op false : -> Bool;

axiom reflexivity(
  T : Sort,
  x : T
)
  |- x = x;

rule rewrite_r(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T -> Prop
)
  |- P(a)
  ------------------------
  |- P(b)
end;

rule rewrite_l(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T -> Prop
)
  |- P(b)
  ------------------------
  |- P(a)
end;

rule symmetry(
  T : Sort,
  x y : T
)
  |- x = y
  ------------------------
  |- y = x
end;

rule transitivity(
  T : Sort,
  x y z : T
)
  |- x = y;
  |- y = z
  ------------------------
  |- x = z
end;

rule and_intro(
  P Q : Prop
)
  |- P;
  |- Q
  ------------------------
  |- P /\ Q
end;

rule and_left(
  P Q : Prop
)
  |- P /\ Q
  ------------------------
  |- P
end;

rule and_right(
  P Q : Prop
)
  |- P /\ Q
  ------------------------
  |- Q
end;

rule or_intro_left(
  P Q : Prop
)
  |- P
  ------------------------
  |- P \/ Q
end;

rule or_intro_right(
  P Q : Prop
)
  |- Q
  ------------------------
  |- P \/ Q
end;

rule or_elim(
  P Q R : Prop
)
  |- P \/ Q;
  P := P |- R;
  Q := Q |- R
  ------------------------
  |- R
end;

rule implication_intro(
  P Q : Prop
)
  P := P |- Q
  ------------------------
  |- P => Q
end;

rule implication_elim(
  P Q : Prop
)
  |- P => Q;
  |- P
  ------------------------
  |- Q
end;

rule negation_intro(
  P : Prop
)
  P := P |- False
  ------------------------
  |- ~P
end;

rule negation_elim(
  P : Prop
)
  |- P;
  |- ~P
  ------------------------
  |- False
end;

rule false_elim(
  P : Prop
)
  |- False
  ------------------------
  |- P
end;

rule biconditional_intro(
  P Q : Prop
)
  |- P => Q;
  |- Q => P
  ------------------------
  |- P <=> Q
end;

rule biconditional_elim_left(
  P Q : Prop
)
  |- P <=> Q
  ------------------------
  |- P => Q
end;

rule biconditional_elim_right(
  P Q : Prop
)
  |- P <=> Q
  ------------------------
  |- Q => P
end;

rule instantiation(
  T : Sort,
  P : T -> Prop
)
  |- forall (y : T) st P(y)
  ------------------------
  |- P(x)
end;

rule generalization(
  T : Sort,
  x : T,
  P : T -> Prop
)
  |- P(x)
  ------------------------
  |- forall (x : T) st P(x)
end;

rule exists_intro(
  T : Sort,
  P : T -> Prop,
  x : T
)
  |- P(x)
  ------------------------
  |- exists (x : T) st P(x)
end;

rule exists_elim(
  T : Sort,
  P : T -> Prop,
  Q : Prop
)
  |- exists (x : T) st P(x);
  x : T, witness := P(x) |- Q
  ------------------------
  |- Q
end;
```

---

# 6. Standard Module: adt.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  rewrite_l,
  transitivity,
  and_intro,
  and_left,
  and_right,
  or_intro_left,
  or_intro_right,
  or_elim
);

sort Pair : Sort * Sort -> Sort;

op pair : A * B -> Pair(A, B);

op fst : Pair(A, B) -> A;

op snd : Pair(A, B) -> B;

axiom fst_pair(
  A B : Sort,
  x : A,
  y : B
)
  |- fst(pair(x, y)) = x;

axiom snd_pair(
  A B : Sort,
  x : A,
  y : B
)
  |- snd(pair(x, y)) = y;

rule pair_cases(
  A B : Sort,
  p : Pair(A, B),
  P : Pair(A, B) -> Prop
)
  x : A, y : B |- P(pair(x, y))
  ------------------------
  |- P(p)
end;

rule product_reflect_intro(
  P Q : Prop
)
  |- P;
  |- Q
  ------------------------
  |- P /\ Q
end;

rule product_reflect_left(
  P Q : Prop
)
  |- P /\ Q
  ------------------------
  |- P
end;

rule product_reflect_right(
  P Q : Prop
)
  |- P /\ Q
  ------------------------
  |- Q
end;

sort Sum : Sort * Sort -> Sort;

op inl : A -> Sum(A, B);

op inr : B -> Sum(A, B);

rule sum_cases(
  A B : Sort,
  s : Sum(A, B),
  P : Sum(A, B) -> Prop
)
  x : A |- P(inl(x));
  y : B |- P(inr(y))
  ------------------------
  |- P(s)
end;

rule sum_reflect_left(
  P Q : Prop
)
  |- P
  ------------------------
  |- P \/ Q
end;

rule sum_reflect_right(
  P Q : Prop
)
  |- Q
  ------------------------
  |- P \/ Q
end;

rule sum_reflect_elim(
  P Q R : Prop
)
  |- P \/ Q;
  P := P |- R;
  Q := Q |- R
  ------------------------
  |- R
end;
```

Note: built-in type syntax `A * B` and `A | B` exists independently from the named `Pair(A, B)` and `Sum(A, B)` sorts. The named sorts are provided for explicit constructor-based reasoning.

---

# 7. Standard Module: monad.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  rewrite_l,
  transitivity
);

theory Functor(
  A B C : Sort,
  F : Sort -> Sort,
  map : (A -> B) * F(A) -> F(B)
) laws
  law identity(
    x : F(A)
  )
    |- map(lambda (a : A) st a, x) = x;

  law composition(
    f : A -> B,
    g : B -> C,
    x : F(A)
  )
    |- map(
         lambda (a : A) st g(f(a)),
         x
       )
       =
       map(
         g,
         map(f, x)
       );
qed;

theory Applicative(
  A B C : Sort,
  F : Sort -> Sort,
  pure : A -> F(A),
  ap : F(A -> B) * F(A) -> F(B)
) laws
  law identity(
    v : F(A)
  )
    |- ap(pure(lambda (x : A) st x), v) = v;

  law homomorphism(
    f : A -> B,
    x : A
  )
    |- ap(pure(f), pure(x)) = pure(f(x));

  law interchange(
    u : F(A -> B),
    y : A
  )
    |- ap(u, pure(y))
       =
       ap(
         pure(lambda (f : A -> B) st f(y)),
         u
       );

  law composition(
    u : F(B -> C),
    v : F(A -> B),
    w : F(A)
  )
    |- ap(
         ap(
           ap(
             pure(lambda (f : B -> C) st lambda (g : A -> B) st lambda (x : A) st f(g(x))),
             u
           ),
           v
         ),
         w
       )
       =
       ap(u, ap(v, w));
qed;

theory Monad(
  A B C : Sort,
  M : Sort -> Sort,
  return : A -> M(A),
  bind : M(A) * (A -> M(B)) -> M(B)
) laws
  law left_identity(
    x : A,
    f : A -> M(B)
  )
    |- bind(return(x), f) = f(x);

  law right_identity(
    m : M(A)
  )
    |- bind(m, return) = m;

  law associativity(
    m : M(A),
    f : A -> M(B),
    g : B -> M(C)
  )
    |- bind(bind(m, f), g)
       =
       bind(
         m,
         lambda (x : A) st bind(f(x), g)
       );
qed;
```

---

# 8. Standard Module: option.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  transitivity
);

import monad(
  Monad
);

sort None : Sort;

sort Option : Sort -> Sort;

op none : -> None;

op some : A -> Option(A);

op return : A -> Option(A);

op bind : Option(A) * (A -> Option(B)) -> Option(B);

axiom return_def(
  A : Sort,
  x : A
)
  |- return(x) = some(x);

axiom bind_none(
  A B : Sort,
  f : A -> Option(B)
)
  |- bind(none, f) = none;

axiom bind_some(
  A B : Sort,
  x : A,
  f : A -> Option(B)
)
  |- bind(some(x), f) = f(x);

rule option_cases(
  A : Sort,
  m : Option(A),
  P : Option(A) -> Prop
)
  |- P(none);
  x : A |- P(some(x))
  ------------------------
  |- P(m)
end;

model OptionMonad satisfies Monad(
  A,
  B,
  C,
  Option,
  return,
  bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      Option(A),
      return(x),
      some(x),
      return_def(A, x),
      lambda (o : Option(A)) st bind(o, f) = f(x)
    )
    case
      A B : Sort;
      x : A;
      f : A -> Option(B);
      |- bind(some(x), f) = f(x);
    proof
      by bind_some(A, B, x, f);
    qed;
  qed;

  law right_identity;
  proof
    by option_cases(
      A,
      m,
      lambda (o : Option(A)) st bind(o, return) = o
    ) cases
      case
        A : Sort;
        |- bind(none, return) = none;
      proof
        by bind_none(A, A, return);
      qed;

      case
        A : Sort;
        x : A;
        |- bind(some(x), return) = some(x);
      proof
        by rewrite_r(
          Option(A),
          bind(some(x), return),
          return(x),
          bind_some(A, A, x, return),
          lambda (o : Option(A)) st o = some(x)
        )
        case
          A : Sort;
          x : A;
          |- return(x) = some(x);
        proof
          by return_def(A, x);
        qed;
      qed;
    qed;
  qed;

  law associativity;
  proof
    by option_cases(
      A,
      m,
      lambda (o : Option(A)) st
        bind(bind(o, f), g)
        =
        bind(
          o,
          lambda (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C : Sort;
        f : A -> Option(B);
        g : B -> Option(C);
        |- bind(bind(none, f), g)
           =
           bind(
             none,
             lambda (x : A) st bind(f(x), g)
           );
      proof
        by rewrite_r(
          Option(B),
          bind(none, f),
          none,
          bind_none(A, B, f),
          lambda (r : Option(B)) st
            bind(r, g)
            =
            bind(
              none,
              lambda (x : A) st bind(f(x), g)
            )
        )
        case
          A B C : Sort;
          f : A -> Option(B);
          g : B -> Option(C);
          |- bind(none, g)
             =
             bind(
               none,
               lambda (x : A) st bind(f(x), g)
             );
        proof
          by bind_none(
            A,
            C,
            lambda (x : A) st bind(f(x), g)
          );
        qed;
      qed;

      case
        A B C : Sort;
        x : A;
        f : A -> Option(B);
        g : B -> Option(C);
        |- bind(bind(some(x), f), g)
           =
           bind(
             some(x),
             lambda (y : A) st bind(f(y), g)
           );
      proof
        by rewrite_r(
          Option(B),
          bind(some(x), f),
          f(x),
          bind_some(A, B, x, f),
          lambda (r : Option(B)) st
            bind(r, g)
            =
            bind(
              some(x),
              lambda (y : A) st bind(f(y), g)
            )
        )
        case
          A B C : Sort;
          x : A;
          f : A -> Option(B);
          g : B -> Option(C);
          |- bind(f(x), g)
             =
             bind(
               some(x),
               lambda (y : A) st bind(f(y), g)
             );
        proof
          by bind_some(
            A,
            C,
            x,
            lambda (y : A) st bind(f(y), g)
          );
        qed;
      qed;
    qed;
  qed;
qed;
```

---

# 9. Standard Module: result.alg

```alg
import core(
  reflexivity,
  rewrite_r
);

import monad(
  Monad
);

sort Result : Sort * Sort -> Sort;

op ok : A -> Result(A, E);

op err : E -> Result(A, E);

op return : A -> Result(A, E);

op bind : Result(A, E) * (A -> Result(B, E)) -> Result(B, E);

axiom return_def(
  A E : Sort,
  x : A
)
  |- return(x) = ok(x);

axiom bind_ok(
  A B E : Sort,
  x : A,
  f : A -> Result(B, E)
)
  |- bind(ok(x), f) = f(x);

axiom bind_err(
  A B E : Sort,
  e : E,
  f : A -> Result(B, E)
)
  |- bind(err(e), f) = err(e);

rule result_cases(
  A E : Sort,
  r : Result(A, E),
  P : Result(A, E) -> Prop
)
  x : A |- P(ok(x));
  e : E |- P(err(e))
  ------------------------
  |- P(r)
end;

model ResultMonad satisfies Monad(
  A,
  B,
  C,
  lambda (X : Sort) st Result(X, E),
  return,
  bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      Result(A, E),
      return(x),
      ok(x),
      return_def(A, E, x),
      lambda (r : Result(A, E)) st bind(r, f) = f(x)
    )
    case
      A B E : Sort;
      x : A;
      f : A -> Result(B, E);
      |- bind(ok(x), f) = f(x);
    proof
      by bind_ok(A, B, E, x, f);
    qed;
  qed;

  law right_identity;
  proof
    by result_cases(
      A,
      E,
      m,
      lambda (r : Result(A, E)) st bind(r, return) = r
    ) cases
      case
        A E : Sort;
        x : A;
        |- bind(ok(x), return) = ok(x);
      proof
        by rewrite_r(
          Result(A, E),
          bind(ok(x), return),
          return(x),
          bind_ok(A, A, E, x, return),
          lambda (r : Result(A, E)) st r = ok(x)
        )
        case
          A E : Sort;
          x : A;
          |- return(x) = ok(x);
        proof
          by return_def(A, E, x);
        qed;
      qed;

      case
        A E : Sort;
        e : E;
        |- bind(err(e), return) = err(e);
      proof
        by bind_err(A, A, E, e, return);
      qed;
    qed;
  qed;

  law associativity;
  proof
    by result_cases(
      A,
      E,
      m,
      lambda (r : Result(A, E)) st
        bind(bind(r, f), g)
        =
        bind(
          r,
          lambda (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C E : Sort;
        x : A;
        f : A -> Result(B, E);
        g : B -> Result(C, E);
        |- bind(bind(ok(x), f), g)
           =
           bind(
             ok(x),
             lambda (y : A) st bind(f(y), g)
           );
      proof
        by rewrite_r(
          Result(B, E),
          bind(ok(x), f),
          f(x),
          bind_ok(A, B, E, x, f),
          lambda (r : Result(B, E)) st
            bind(r, g)
            =
            bind(
              ok(x),
              lambda (y : A) st bind(f(y), g)
            )
        )
        case
          A B C E : Sort;
          x : A;
          f : A -> Result(B, E);
          g : B -> Result(C, E);
          |- bind(f(x), g)
             =
             bind(
               ok(x),
               lambda (y : A) st bind(f(y), g)
             );
        proof
          by bind_ok(
            A,
            C,
            E,
            x,
            lambda (y : A) st bind(f(y), g)
          );
        qed;
      qed;

      case
        A B C E : Sort;
        e : E;
        f : A -> Result(B, E);
        g : B -> Result(C, E);
        |- bind(bind(err(e), f), g)
           =
           bind(
             err(e),
             lambda (x : A) st bind(f(x), g)
           );
      proof
        by rewrite_r(
          Result(B, E),
          bind(err(e), f),
          err(e),
          bind_err(A, B, E, e, f),
          lambda (r : Result(B, E)) st
            bind(r, g)
            =
            bind(
              err(e),
              lambda (x : A) st bind(f(x), g)
            )
        )
        case
          A B C E : Sort;
          e : E;
          f : A -> Result(B, E);
          g : B -> Result(C, E);
          |- bind(err(e), g)
             =
             bind(
               err(e),
               lambda (x : A) st bind(f(x), g)
             );
        proof
          by bind_err(
            A,
            C,
            E,
            e,
            lambda (x : A) st bind(f(x), g)
          );
        qed;
      qed;
    qed;
  qed;
qed;
```

---

# 10. Standard Module: list.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  transitivity
);

import monad(
  Monad
);

sort List : Sort -> Sort;

op nil : -> List(A);

op cons : A * List(A) -> List(A);

op append : List(A) * List(A) -> List(A);

op singleton : A -> List(A);

op return : A -> List(A);

op bind : List(A) * (A -> List(B)) -> List(B);

axiom append_nil_left(
  A : Sort,
  xs : List(A)
)
  |- append(nil, xs) = xs;

axiom append_cons_left(
  A : Sort,
  x : A,
  xs ys : List(A)
)
  |- append(cons(x, xs), ys) = cons(x, append(xs, ys));

axiom singleton_def(
  A : Sort,
  x : A
)
  |- singleton(x) = cons(x, nil);

axiom return_def(
  A : Sort,
  x : A
)
  |- return(x) = singleton(x);

axiom bind_nil(
  A B : Sort,
  f : A -> List(B)
)
  |- bind(nil, f) = nil;

axiom bind_cons(
  A B : Sort,
  x : A,
  xs : List(A),
  f : A -> List(B)
)
  |- bind(cons(x, xs), f) = append(f(x), bind(xs, f));

axiom bind_singleton(
  A B : Sort,
  x : A,
  f : A -> List(B)
)
  |- bind(singleton(x), f) = f(x);

axiom append_nil_right(
  A : Sort,
  xs : List(A)
)
  |- append(xs, nil) = xs;

axiom append_associativity(
  A : Sort,
  xs ys zs : List(A)
)
  |- append(append(xs, ys), zs) = append(xs, append(ys, zs));

axiom bind_append(
  A B : Sort,
  xs ys : List(A),
  f : A -> List(B)
)
  |- bind(append(xs, ys), f) = append(bind(xs, f), bind(ys, f));

rule list_induction(
  A : Sort,
  xs : List(A),
  P : List(A) -> Prop
)
  |- P(nil);
  x : A, rest : List(A), ih := P(rest) |- P(cons(x, rest))
  ------------------------
  |- P(xs)
end;

model ListMonad satisfies Monad(
  A,
  B,
  C,
  List,
  return,
  bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      List(A),
      return(x),
      singleton(x),
      return_def(A, x),
      lambda (xs : List(A)) st bind(xs, f) = f(x)
    )
    case
      A B : Sort;
      x : A;
      f : A -> List(B);
      |- bind(singleton(x), f) = f(x);
    proof
      by bind_singleton(A, B, x, f);
    qed;
  qed;

  law right_identity;
  proof
    by list_induction(
      A,
      m,
      lambda (xs : List(A)) st bind(xs, return) = xs
    ) cases
      case
        A : Sort;
        |- bind(nil, return) = nil;
      proof
        by bind_nil(A, A, return);
      qed;

      case
        A : Sort;
        x : A;
        rest : List(A);
        ih := bind(rest, return) = rest;
        |- bind(cons(x, rest), return) = cons(x, rest);
      proof
        by rewrite_r(
          List(A),
          bind(cons(x, rest), return),
          append(return(x), bind(rest, return)),
          bind_cons(A, A, x, rest, return),
          lambda (ys : List(A)) st ys = cons(x, rest)
        )
        case
          A : Sort;
          x : A;
          rest : List(A);
          ih := bind(rest, return) = rest;
          |- append(return(x), bind(rest, return)) = cons(x, rest);
        proof
          by rewrite_r(
            List(A),
            return(x),
            singleton(x),
            return_def(A, x),
            lambda (ys : List(A)) st append(ys, bind(rest, return)) = cons(x, rest)
          )
          case
            A : Sort;
            x : A;
            rest : List(A);
            ih := bind(rest, return) = rest;
            |- append(singleton(x), bind(rest, return)) = cons(x, rest);
          proof
            by rewrite_r(
              List(A),
              bind(rest, return),
              rest,
              ih,
              lambda (ys : List(A)) st append(singleton(x), ys) = cons(x, rest)
            )
            case
              A : Sort;
              x : A;
              rest : List(A);
              |- append(singleton(x), rest) = cons(x, rest);
            proof
              by reflexivity(List(A), cons(x, rest));
            qed;
          qed;
        qed;
      qed;
    qed;
  qed;

  law associativity;
  proof
    by list_induction(
      A,
      m,
      lambda (xs : List(A)) st
        bind(bind(xs, f), g)
        =
        bind(
          xs,
          lambda (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C : Sort;
        f : A -> List(B);
        g : B -> List(C);
        |- bind(bind(nil, f), g)
           =
           bind(
             nil,
             lambda (x : A) st bind(f(x), g)
           );
      proof
        by rewrite_r(
          List(B),
          bind(nil, f),
          nil,
          bind_nil(A, B, f),
          lambda (ys : List(B)) st
            bind(ys, g)
            =
            bind(
              nil,
              lambda (x : A) st bind(f(x), g)
            )
        )
        case
          A B C : Sort;
          f : A -> List(B);
          g : B -> List(C);
          |- bind(nil, g)
             =
             bind(
               nil,
               lambda (x : A) st bind(f(x), g)
             );
        proof
          by bind_nil(
            A,
            C,
            lambda (x : A) st bind(f(x), g)
          );
        qed;
      qed;

      case
        A B C : Sort;
        x : A;
        rest : List(A);
        ih := bind(bind(rest, f), g) = bind(rest, lambda (x : A) st bind(f(x), g));
        f : A -> List(B);
        g : B -> List(C);
        |- bind(bind(cons(x, rest), f), g)
           =
           bind(
             cons(x, rest),
             lambda (y : A) st bind(f(y), g)
           );
      proof
        by rewrite_r(
          List(B),
          bind(cons(x, rest), f),
          append(f(x), bind(rest, f)),
          bind_cons(A, B, x, rest, f),
          lambda (ys : List(B)) st
            bind(ys, g)
            =
            bind(
              cons(x, rest),
              lambda (y : A) st bind(f(y), g)
            )
        )
        case
          A B C : Sort;
          x : A;
          rest : List(A);
          ih := bind(bind(rest, f), g) = bind(rest, lambda (x : A) st bind(f(x), g));
          f : A -> List(B);
          g : B -> List(C);
          |- bind(append(f(x), bind(rest, f)), g)
             =
             bind(
               cons(x, rest),
               lambda (y : A) st bind(f(y), g)
             );
        proof
          by rewrite_r(
            List(C),
            bind(append(f(x), bind(rest, f)), g),
            append(bind(f(x), g), bind(bind(rest, f), g)),
            bind_append(B, C, f(x), bind(rest, f), g),
            lambda (zs : List(C)) st
              zs
              =
              bind(
                cons(x, rest),
                lambda (y : A) st bind(f(y), g)
              )
          )
          case
            A B C : Sort;
            x : A;
            rest : List(A);
            ih := bind(bind(rest, f), g) = bind(rest, lambda (x : A) st bind(f(x), g));
            f : A -> List(B);
            g : B -> List(C);
            |- append(bind(f(x), g), bind(bind(rest, f), g))
               =
               bind(
                 cons(x, rest),
                 lambda (y : A) st bind(f(y), g)
               );
          proof
            by rewrite_r(
              List(C),
              bind(bind(rest, f), g),
              bind(rest, lambda (x : A) st bind(f(x), g)),
              ih,
              lambda (zs : List(C)) st
                append(bind(f(x), g), zs)
                =
                bind(
                  cons(x, rest),
                  lambda (y : A) st bind(f(y), g)
                )
            )
            case
              A B C : Sort;
              x : A;
              rest : List(A);
              f : A -> List(B);
              g : B -> List(C);
              |- append(
                   bind(f(x), g),
                   bind(rest, lambda (x : A) st bind(f(x), g))
                 )
                 =
                 bind(
                   cons(x, rest),
                   lambda (y : A) st bind(f(y), g)
                 );
            proof
              by bind_cons(
                A,
                C,
                x,
                rest,
                lambda (y : A) st bind(f(y), g)
              );
            qed;
          qed;
        qed;
      qed;
    qed;
  qed;
qed;
```

---

# 11. Standard Module: nat.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  transitivity
);

sort Nat : Sort;

op 0 : -> Nat;

op s : Nat -> Nat;

op + : Nat * Nat -> Nat;

op * : Nat * Nat -> Nat;

axiom add_zero_left(
  n : Nat
)
  |- 0 + n = n;

axiom add_succ_left(
  n m : Nat
)
  |- s(n) + m = s(n + m);

axiom mul_zero_left(
  n : Nat
)
  |- 0 * n = 0;

axiom mul_succ_left(
  n m : Nat
)
  |- s(n) * m = m + (n * m);

rule induction(
  P : Nat -> Prop
)
  |- P(0);
  n : Nat;
  ih := P(n);
  |- P(s(n))
  ------------------------
  |- forall (n : Nat) st P(n)
end;

lemma add_zero_right
  |- forall (n : Nat) st n + 0 = n;
proof
  by induction(
    lambda (k : Nat) st k + 0 = k
  ) cases
    case
      |- 0 + 0 = 0;
    proof
      by add_zero_left(0);
    qed;

    case
      k : Nat;
      ih := k + 0 = k;
      |- s(k) + 0 = s(k);
    proof
      by rewrite_r(
        Nat,
        k + 0,
        k,
        ih,
        lambda (x : Nat) st s(k) + 0 = s(x)
      )
      case
        k : Nat;
        ih := k + 0 = k;
        |- s(k) + 0 = s(k + 0);
      proof
        by add_succ_left(k, 0);
      qed;
    qed;
  qed;
qed;
```

---

# 12. Standard Module: group.alg

```alg
import core(
  reflexivity,
  rewrite_r,
  transitivity,
  symmetry
);

theory Magma(
  S : Sort,
  op : S * S -> S
) laws
  law closure(
    x y : S
  )
    |- op(x, y) = op(x, y);
qed;

theory Semigroup(
  S : Sort,
  op : S * S -> S
) laws
  include Magma(S, op);

  law associativity(
    x y z : S
  )
    |- op(op(x, y), z) = op(x, op(y, z));
qed;

theory Monoid(
  S : Sort,
  op : S * S -> S,
  e : S
) laws
  include Semigroup(S, op);

  law left_identity(
    x : S
  )
    |- op(e, x) = x;

  law right_identity(
    x : S
  )
    |- op(x, e) = x;
qed;

theory CommutativeMonoid(
  S : Sort,
  op : S * S -> S,
  e : S
) laws
  include Monoid(S, op, e);

  law commutativity(
    x y : S
  )
    |- op(x, y) = op(y, x);
qed;

theory Group(
  S : Sort,
  op : S * S -> S,
  e : S,
  inv : S -> S
) laws
  include Monoid(S, op, e);

  law left_inverse(
    x : S
  )
    |- op(inv(x), x) = e;

  law right_inverse(
    x : S
  )
    |- op(x, inv(x)) = e;
qed;

theory AbelianGroup(
  S : Sort,
  op : S * S -> S,
  e : S,
  inv : S -> S
) laws
  include Group(S, op, e, inv);

  law commutativity(
    x y : S
  )
    |- op(x, y) = op(y, x);
qed;
```

---

# 13. Complete Construct Semantics Summary

## `import`

Loads another module and makes its symbols available by qualification.

Selected imported symbols may also be made available unqualified or through aliases.

## `sort`

Declares a sort or sort constructor.

## `op`

Declares a total operator.

## `axiom`

Declares an assumed proposition.

## `rule`

Declares an inference rule.

## `lemma`

Declares and proves a reusable proposition.

## `theorem`

Declares and proves a reusable proposition.

## `proof`

Starts a proof block.

## `qed;`

Ends a proof block.

## `by`

Applies an axiom, lemma, theorem, local proof binding, or rule.

## `case`

Provides a proof for one generated premise.

## `theory`

Declares a parameterized collection of laws.

## `include`

Inside a theory, imports the law obligations of another theory by inheritance.

## `law`

Inside a theory, declares a required property.

Inside a model, selects a law obligation and provides a proof.

## `model`

Declares that a concrete interpretation satisfies a theory.

## `satisfies`

Connects a model to the theory it models.

## `iff`

Introduces the evidence block proving satisfaction.

---

# 14. Checker Completeness Checklist

A complete Algae v2 implementation must include:

1. Lexer.
2. Parser.
3. AST construction.
4. Module loader.
5. Import resolver.
6. Qualified-name resolver.
7. Alias resolver.
8. Kind checker.
9. Type checker.
10. Proposition checker.
11. Sequent checker.
12. Rule well-formedness checker.
13. Axiom environment.
14. Lemma environment.
15. Theorem environment.
16. Theory environment.
17. Law environment.
18. Model environment.
19. Include expansion for theories.
20. Transitive law collection for models.
21. Proof obligation generation for rule applications.
22. Proof obligation generation for model laws.
23. Proof block checker.
24. Case checker.
25. Context checker.
26. Proof binding resolver.
27. Side-condition checker for generalization.
28. Verification that every model law is proven exactly once.
29. Verification that no unknown model law is proven.
30. Verification that all terms in proofs typecheck.
31. Verification that all propositions in proofs typecheck.
32. Verification that every final proof closes the current goal.

# 15. Tree-sitter and editors

Tree-sitter highlight live in editors/tree-sitter

## Neovim

The neovim plugin using the tree-sitter highligh lives in editors/neovim/

---

# 16. Style

Stylistic conventions for writing Algae source. These are recommendations for
readability; they are not enforced by the parser.

1. **Avoid blank lines inside rules.** Keep a `rule`'s premises, separator line,
   and conclusion on consecutive lines so the whole inference reads as a single
   unit.

   Preferred:

   ```alg
   rule transitivity(
     T : Sort,
     x y z : T
   )
     |- x = y;
     |- y = z
     ------------------------
     |- x = z
   end;
   ```

   Avoid:

   ```alg
   rule transitivity(
     T : Sort,
     x y z : T
   )
     |- x = y;

     |- y = z

     ------------------------

     |- x = z
   end;
   ```
