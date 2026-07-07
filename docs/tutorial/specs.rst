=================================
Sorts, operations, and equations
=================================

So far the propositions were abstract — bare ``A``, ``B : Prop``. Now let's give
ourselves something concrete to talk *about*. That's the **algebraic
specification** side of Algae: you name a little world and pin down how it
behaves, then prove things in it. This chapter is a brief tour; the later
chapters (:doc:`worlds`, :doc:`stack`, :doc:`theories`) go deeper.

Sorts and operations
====================

A **sort** is a base type. An **operation** (``op``) is a function symbol with a
signature — the arrow ``→`` gives the result type, ``*`` builds a tuple of
arguments. Here's the opening of the standard library's ``nat``:

.. code-block:: alg

   sort Nat : Sort;            # a base sort

   op 0 : → Nat;               # a constant (no arguments, so just → Nat)
   op s : Nat → Nat;           # successor: takes a Nat, returns a Nat
   op + : Nat * Nat → Nat;     # addition: takes two Nats, written infix as x + y

That's a *vocabulary* and nothing more. ``0``, ``s``, and ``+`` are just symbols;
nothing yet says ``s(0)`` is "one" or that ``+`` adds. In Algae, operators are
**inert** — the checker never evaluates them. ``0 + 0`` does not quietly become
``0``.

Axioms give operators meaning
=============================

Operators earn their meaning from **axioms** — sequents asserted true with no
proof. ``nat`` gives ``+`` its personality with two equations:

.. code-block:: alg

   axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
   axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);

An axiom is exactly a **zero-premise rule** (recall :doc:`backward-reasoning`):
nothing to establish, so wherever its conclusion matches your goal, it closes it.
Describing a structure by its operations plus a handful of such equations is what
"algebraic specification" means — and because the axioms are all you assume, a
proof from them holds for *every* structure that satisfies them.

Equational reasoning
====================

Because ``add_zero_left`` is an equation whose conclusion is ``0 + n = n``, we can
close any goal that matches it. Instantiate at ``n = 0`` and the conclusion reads
``0 + 0 = 0`` — precisely a goal we might have:

.. code-block:: alg

   import nat;

   lemma zero_plus_zero
     ⊢ 0 + 0 = 0;
   proof
     by add_zero_left(0);
   qed;

One ``by``, and it closes — the axiom is premise-free. This is *equational
reasoning*: you prove things by lining goals up with equations you've assumed
(or already proved).

.. admonition:: Definitional equality is α/β only
   :class: note

   There's one rule newcomers trip on. Algae's built-in notion of "the same
   term" — **definitional equality** — is *α/β-equivalence only*: two terms are
   the same when they share a normal form after renaming bound variables and
   applying lambdas, full stop. Operators are inert, so ``0 + 0`` is **not**
   definitionally ``0``. That's why ``zero_plus_zero`` needs ``add_zero_left`` and
   can't be closed by "they're obviously equal." Every use of an equation is a
   step you can point to — there is no hidden computation.

Applying an equation to a *part* of a goal — rewriting ``0 + 0`` to ``0`` buried
*inside* a larger proposition, rather than matching the whole thing — is the job
of the next chapter: reflexivity and the rewrite rules.
