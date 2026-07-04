===========================
Two worlds and a vocabulary
===========================

Hold one idea in your head from the very start, because it explains most of
Algae's shape: there are **two separate worlds**, and their names live in **two
disjoint namespaces**.

- The **term world** — sorts, operators, variables, and the propositions built
  from them. This is the *term namespace*.
- The **proof world** — axioms, rules, lemmas, and hypotheses: the things you
  *apply* to build a proof. This is the *proof namespace*.

A name in one world is invisible to the other. You can't sneak a lemma into a
proposition, and you can't apply an operator as a proof step. It feels strict at
first, but it's exactly what keeps proofs honest — and we'll make it delightfully
concrete back in :doc:`induction`.

Running the checker
===================

Everything below is a ``.alg`` file you feed to the CLI:

.. code-block:: sh

   cargo run -p algae-cli -- verify file.alg      # elaborate + proof-check
   cargo run -p algae-cli -- typecheck file.alg   # signatures only, skip proofs
   cargo run -p algae-cli -- parse file.alg        # syntax only
   cargo run -p algae-cli -- fmt file.alg          # normalize operator glyphs

``verify`` is the one that runs the proof checker. A clean run prints
``… : checked N proof obligation(s)`` — the sound of success.

ASCII or Unicode, your call
---------------------------

Every operator has an ASCII and a Unicode spelling, and both lex to the same
token. This tutorial uses the pretty Unicode forms; if you'd rather type ASCII,
``fmt`` converts it to Unicode for you (and ``fmt --ascii`` converts back).

.. list-table::
   :header-rows: 1

   * - ASCII
     - Unicode
     - meaning
   * - ``|-``
     - ``⊢``
     - turnstile (a sequent)
   * - ``->``
     - ``→``
     - function type
   * - ``forall``
     - ``∀``
     - universal
   * - ``exists``
     - ``∃``
     - existential
   * - ``lambda``
     - ``λ``
     - lambda
   * - ``=>``
     - ``⇒``
     - implication
   * - ``/\`` ``\/`` ``~``
     - ``∧`` ``∨`` ``¬``
     - and, or, not

The product type is always written ``*`` (as in ``Nat * Nat``).

Sorts, operators, types
=======================

A **sort** is a base type. An **operator** is a total function symbol with a
signature. Nothing here is a proof yet — this is the term world's vocabulary.
Here's the opening of ``nat.alg``:

.. code-block:: alg

   sort Nat : Sort;            # a base sort

   op 0 : → Nat;              # a nullary operator (a constant)
   op s : Nat → Nat;          # successor
   op + : Nat * Nat → Nat;    # a binary operator, written infix as x + y

Types are built from sorts with ``*`` (product), ``|`` (sum), and ``→``
(function). A proposition has the special type ``Prop``; a predicate is therefore
just an operator into ``Prop``, e.g. ``op even : Nat → Prop``. ``option.alg`` shows
all three at once — its ``bind`` takes a product of an ``Option(A)`` and a
function:

.. code-block:: alg

   op bind : Option(A) * (A → Option(B)) → Option(B);

Propositions and sequents
=========================

A **proposition** is just a ``Prop``-valued term: an equation ``a = b``, a
connective (``∧``, ``∨``, ``⇒``, ``⇔``, ``¬``), a quantifier (``∀``, ``∃``), or a
predicate applied to arguments. Terms and propositions share one grammar.

A **sequent** is a proposition under a context of assumptions::

   context ⊢ proposition

The context lists typed variables and named hypotheses. With an empty context you
just write ``⊢ proposition``. These two read "``x = x``" and "under ``x`` and
``y``, and a proof of ``x = y``, conclude ``y = x``":

.. code-block:: alg

   ⊢ x = x
   x : Nat, y : Nat, h := x = y ⊢ y = x

A **lemma** states a sequent and must supply a proof of it. That's next.
