==============================
A tour of the standard library
==============================

You've met the moving parts one at a time. Now let's take them for a proper walk.
The standard library is where Algae keeps its **inference rules** — the verbs of
proving. Every proof you'll ever write is built by chaining these, so it pays to
know the cast.

This tour is a guided course, not a reference card. We'll go module by module,
meet each rule, read what it *means*, watch it work in a couple of proofs, and
then hand you an unfinished one to complete. The editors are live — finish the
proof, press **Check ▶**, and the kernel will tell you if you've got it.

Everything here is real: the rules live in the modules linked below (they open in
a new tab, so keep one handy while you read).

How to read a rule
==================

Every rule has the same anatomy — premises above a line, a conclusion below:

.. code-block:: alg

   rule and_intro(P Q : Prop)
     ⊢ P;
     ⊢ Q
     ────────────────────────
     ⊢ P ∧ Q
   end;

To *apply* a rule with ``by``, the kernel matches your current goal against its
**conclusion** and hands you back one subgoal per **premise**. So the number of
premises decides the shape of the step (you saw this in :doc:`../backward-reasoning`):

- **zero premises** — the rule closes the goal outright (``by refl(Nat, n);``).
- **one premise** — continue in the same block with ``then`` (``by symmetry(…)
  then ⊢ …; by …``).
- **two or more** — branch with ``cases``, one ``case`` per premise.

The arguments in ``by rule(args)`` fill the rule's *parameters* — the ``P``, ``Q``,
``T``, ``x`` … — and are matched and typechecked just like operator arguments. When
in doubt about what a rule wants, leave a hole (``by rule?;``) and let the checker
tell you (:doc:`../editor`).

The modules
===========

- |core.alg| — equality, the logical connectives, and the quantifiers. The
  bedrock; almost every proof imports something from here.
- |nat.alg| — the natural numbers, addition and multiplication, and
  ``induction``.
- |adt.alg| — pairs and sums (``Pair``, ``Sum``) with their case-analysis rules.
- |option.alg|, |result.alg|, |list.alg| — data types, their case rules, and the
  equations that drive their proofs.
- |monad.alg|, |group.alg| — the theories from :doc:`../theories`.

Grab a module link, keep it open, and let's begin.

.. toctree::
   :maxdepth: 1

   logic
   quantifiers
   data
