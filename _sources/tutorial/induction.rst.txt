=====================
Induction and friends
=====================

Time for the real thing. ``nat`` defines addition and an induction rule whose
conclusion is a *universally quantified* proposition:

.. code-block:: alg

   rule induction(P : Nat → Prop)
     ⊢ P(0);
     n : Nat, ih := P(n) ⊢ P(s(n))
     ──────────────────────────────
     ⊢ ∀ (n : Nat) st P(n)
   end;

Two premises — the base case ``P(0)`` and the step case (assume ``ih := P(n)``,
prove ``P(s(n))``) — and a conclusion ``∀ n. P(n)``. Two premises means two
branches, so this is a job for ``cases``. Here is the full proof of
``n + 0 = n``, straight from ``nat.alg``:

.. code-block:: alg

   import core(refl, rewrite_r, transitivity);

   sort Nat : Sort;
   op 0 : → Nat;
   op s : Nat → Nat;
   op + : Nat * Nat → Nat;

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
         by add_zero_left(0);             # conclusion 0 + n = n at n = 0 is the goal
       qed;

       case
         k : Nat;
         ih := k + 0 = k;                 # step: assume P(k)
         ⊢ s(k) + 0 = s(k);             # prove P(s k)
         by rewrite_r(Nat, k + 0, k, ih, s(k) + 0 = s(_))
         then ⊢ s(k) + 0 = s(k + 0);      # goal after rewriting k <- k + 0
         by add_succ_left(k, 0);          # conclusion s(k) + 0 = s(k + 0) is the goal
       qed;
     qed;
   qed;

Reading it as a tree:

- ``by induction(_ + 0 = _)`` supplies the **motive** ``P`` (that ``_ + 0 = _``
  is a hole — more on it in a second). The goal ``∀ n. n + 0 = n`` matches
  ``induction``'s conclusion, producing **two** subgoals, one per ``case``.
- The **base case** ``0 + 0 = 0`` is closed by ``add_zero_left(0)``: its
  conclusion ``0 + n = n``, at ``n = 0``, is exactly ``0 + 0 = 0``.
- The **step case** assumes ``ih := k + 0 = k`` and must prove ``s(k) + 0 = s(k)``.
  It uses ``ih`` to rewrite ``k`` to ``k + 0`` under ``s``, leaving
  ``s(k) + 0 = s(k + 0)``, which ``add_succ_left(k, 0)`` discharges.

Notice ``ih`` — a **hypothesis** introduced by the step case — being handed to
``rewrite_r`` as a proof argument. That's the proof namespace at work: ``ih`` is
not a term, it's *evidence*.

Eigenvariables
==============

In the step case, ``k`` is an **eigenvariable**: a fresh variable standing for an
arbitrary ``Nat``. Introducing it is the formal version of "let ``k`` be
arbitrary." The kernel insists it's genuinely fresh (it may not already occur in
the surrounding context) — and that freshness is exactly what makes "prove
``P(k)`` for arbitrary ``k``" sound as "prove ``∀ k. P(k)``".

The ``_`` shorthand
===================

Writing motives by hand is tedious, so ``_`` is sugar for a lambda. In the proof
above, ``by induction(_ + 0 = _)`` means ``by induction(λ k. k + 0 = k)``: each
``_`` becomes the lambda's bound variable. Reach for ``_`` whenever a predicate
argument is obvious from the goal. (This is a different beast from the ``?name``
holes in :doc:`holes` — ``_`` is filled in silently; ``?`` asks the checker to
*talk to you*.)

Parameters vs. ``forall``
=========================

A lemma can bind a variable two ways. They read as the same theorem but behave
differently in proofs:

.. code-block:: alg

   lemma foo(x : T) ⊢ P(x);                  # a parameter
   lemma foo         ⊢ ∀ (x : T) st P(x);    # a quantifier in the proposition

Both say "P holds for every x." The difference is representation:

- A **parameter** ``x`` is a *schematic* variable — an implicit universal. As a
  proof step, ``by foo(a)`` instantiates it directly, proving ``P(a)`` for any
  term ``a``.
- A **``forall``** puts the universal *inside* the proposition, as an object-level
  connective you introduce and eliminate with explicit rules.

``core`` provides the two bridges between them:

.. code-block:: alg

   rule forall_intro(T : Sort, P : T → Prop)      rule forall_elim(T : Sort, P : T → Prop)
     x : T ⊢ P(x)                                   ⊢ ∀ (y : T) st P(y)
     ────────────────────────                        ────────────────────────
     ⊢ ∀ (x : T) st P(x)                       x : T ⊢ P(x)

``forall_intro`` turns a proof of ``P(x)`` for a fresh eigenvariable ``x`` into
``∀ x. P(x)``; ``forall_elim`` goes the other way. So proving a ``forall`` goal
begins by introducing the variable:

.. code-block:: alg

   import nat;
   import core(refl, forall_intro);

   lemma all_refl
     ⊢ ∀ (n : Nat) st n = n;
   proof
     by forall_intro(Nat, λ (n : Nat) st n = n)
     then n : Nat ⊢ n = n;    # n introduced as an eigenvariable
     by refl(Nat, n);
   qed;

Here the ``then`` keeps its context: ``forall_intro`` introduces the fresh
eigenvariable ``n``, so the continuation names it (``n : Nat ⊢ …``). When a step
introduces no new variables — most ``rewrite_r`` steps — you can drop the context
and just write ``then ⊢ <goal>;``.

This is also why ``induction`` states its conclusion as ``∀ (n : Nat) st P(n)``
rather than taking ``n`` as a parameter: the step case must reason about ``n`` as
a bound eigenvariable the premises discharge, which a caller-supplied parameter
couldn't express.

The two namespaces, made concrete
==================================

Remember the two worlds from the very first page? Here's the promised payoff.
Propositions are elaborated in the **term namespace**; ``by`` references and proof
arguments resolve in the **proof namespace**. They never mix.

The practical consequence: **a proposition cannot mention a proof-former.** Try to
use an axiom's name as if it were a term and the checker stops you cold:

.. code-block:: alg

   # THIS DOES NOT COMPILE
   import nat;

   lemma oops
     ⊢ add_zero_left(0);   # error: `add_zero_left` is an axiom, not a term
   proof
     by add_zero_left(0);
   qed;

``verify`` rejects it with ``error: unbound name add_zero_left``: the
proposition ``add_zero_left(0)`` is elaborated in the term namespace, and there's
no *term* called ``add_zero_left`` — only a proof-namespace axiom. To talk about a
fact inside a proposition you'd declare an operator, e.g. ``op even : Nat → Prop``.
The reverse is blocked too: an operator can't be applied as a tactic in a ``by``.

That's why axioms, rules, and lemmas aren't "first-class values": you can
reference them in proofs, apply them, and pass hypotheses as evidence — but you
can't put them in a proposition or quantify over them.
