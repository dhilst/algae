====================
Propositional logic
====================

Algae is a tool for *proving* things, so before we prove anything we need a
shared idea of what the things we prove are made of. This chapter is a quick
refresher on **classical propositional logic** — the algebra of true/false
statements. There's no editor here and no Algae syntax; it's just the ground the
rest of the tutorial stands on. If it's familiar, skim it.

A **proposition** is a statement that is either **true** (``T``) or **false**
(``F``). Write bare propositions with letters ``A``, ``B``, ``C``. From simple
propositions we build compound ones with four **connectives**, and each
connective is defined completely by a **truth table** — what it does for every
combination of truth values of its parts.

Negation: ``¬`` (not)
=====================

``¬A`` ("not ``A``") flips the truth value: it's true exactly when ``A`` is
false.

.. list-table::
   :header-rows: 1
   :widths: 20 20

   * - ``A``
     - ``¬A``
   * - T
     - F
   * - F
     - T

Conjunction: ``∧`` (and)
========================

``A ∧ B`` ("``A`` and ``B``") is true only when **both** halves are true.

.. list-table::
   :header-rows: 1
   :widths: 15 15 20

   * - ``A``
     - ``B``
     - ``A ∧ B``
   * - T
     - T
     - T
   * - T
     - F
     - F
   * - F
     - T
     - F
   * - F
     - F
     - F

Disjunction: ``∨`` (or)
=======================

``A ∨ B`` ("``A`` or ``B``") is true when **at least one** half is true. This is
the *inclusive* or — ``A ∨ B`` is still true when both are.

.. list-table::
   :header-rows: 1
   :widths: 15 15 20

   * - ``A``
     - ``B``
     - ``A ∨ B``
   * - T
     - T
     - T
   * - T
     - F
     - T
   * - F
     - T
     - T
   * - F
     - F
     - F

Implication: ``→`` (implies)
============================

``A → B`` ("``A`` implies ``B``", or "if ``A`` then ``B``") is the one that
surprises people. It is false in exactly **one** case: when the premise ``A`` is
true but the conclusion ``B`` is false. Whenever ``A`` is false, ``A → B`` is
*true* regardless of ``B`` — a promise with a false premise is never broken.

.. list-table::
   :header-rows: 1
   :widths: 15 15 20

   * - ``A``
     - ``B``
     - ``A → B``
   * - T
     - T
     - T
   * - T
     - F
     - F
   * - F
     - T
     - T
   * - F
     - F
     - T

.. admonition:: A note on spelling
   :class: note

   This chapter uses the conventional logic symbols. Three of them are exactly
   how Algae writes them: ``∧`` (ASCII ``/\``), ``∨`` (ASCII ``\/``), and ``¬``
   (ASCII ``~``). Implication is the exception: Algae reserves ``→`` for
   *function types*, so it writes implication as ``⇒`` (ASCII ``=>``). From the
   next chapter on we use Algae's spellings — read ``⇒`` as the ``→`` from the
   table above.

Why truth tables aren't enough
==============================

Truth tables *decide* propositional logic: to check whether a formula is always
true, you could grind through every row. But two things make that a dead end for
real proofs. First, the tables blow up — ``n`` letters means ``2ⁿ`` rows.
Second, and more importantly, the moment we add **variables and quantifiers**
("for *every* number ``n`` …") there are infinitely many rows and no table to
grind.

So instead of computing truth values, Algae proves statements the way
mathematicians do: by **inference rules** — small, fixed steps like "to prove a
conjunction, prove each half" that chain together into a proof. Those rules, and
how the kernel checks a chain of them, are the subject of the next chapter.
