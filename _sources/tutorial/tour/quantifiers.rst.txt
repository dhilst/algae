=====================
Quantifiers and ``⇔``
=====================

The last stretch of |core.alg| is the quantifiers, ``∀`` and ``∃``. They follow the
same *build one / use one* pattern as the connectives, with one twist: their rules
carry a **motive** — a predicate ``T → Prop`` naming the property you're quantifying.
We'll work over an abstract sort ``T`` and an abstract predicate ``P : T → Prop``, so
nothing is hidden.

For all: ``∀``
==============

To prove ``∀ x. P(x)``, prove ``P(x)`` for a *fresh, arbitrary* ``x``.
``forall_intro`` hands you that eigenvariable — one premise, so ``then``, and the
``then`` carries ``x`` into its context. The one property we can prove of *every*
element with no assumptions is that it equals itself:

.. code-block:: alg

   import core(forall_intro, refl);

   sort T : Sort;

   lemma everything_is_itself
     ⊢ ∀ (x : T) st x = x;
   proof
     by forall_intro(T, _ = _) then x : T ⊢ x = x; by refl(T, x);
   qed;

Here the motive ``_ = _`` is the predicate ``λ (x : T) st x = x`` (that ``_`` sugar
from :doc:`../induction`). Because ``x`` was introduced fresh, proving ``x = x`` for
it counts as proving it for everyone.

To *use* a ``∀``, instantiate it at a specific term. ``forall_elim`` takes
``∀ y. P(y)`` and a point ``a`` and yields ``P(a)``. This time the motive is the
abstract ``P`` itself, and the universal fact is a lemma parameter:

.. code-block:: alg

   import core(forall_elim);

   sort T : Sort;

   lemma at_a_point(P : T → Prop, a : T, all := ∀ (y : T) st P(y))
     ⊢ P(a);
   proof
     by forall_elim(T, P, a) then ⊢ ∀ (y : T) st P(y); by all;
   qed;

The third argument, ``a``, is the point you're instantiating at; the ``then`` goal
is the universal statement you're drawing it from, discharged by ``all``.

There exists: ``∃``
===================

Building a ``∃`` means producing a **witness**. ``exists_intro`` takes a term ``a``
and a proof that the property holds *of that term*:

.. code-block:: alg

   import core(exists_intro);

   sort T : Sort;

   lemma there_is_one(P : T → Prop, a : T, pa := P(a))
     ⊢ ∃ (x : T) st P(x);
   proof
     by exists_intro(T, P, a) then ⊢ P(a); by pa;
   qed;

We offered ``a`` as the witness, so the leftover goal is the property at ``a`` —
``P(a)`` — which our assumption ``pa`` supplies.

*Using* a ``∃`` is the dual of using a ``∨``: you get a witness but you don't get to
know which one, so whatever you conclude must hold no matter who it is.
``exists_elim`` hands you a fresh ``x`` and the hypothesis ``witness := P(x)``
(named, as always, after the rule's premise), and asks you to reach your goal from
there — two premises, ``cases``. To show it really gives you something usable, we
unpack an existential and immediately *repack* it:

.. code-block:: alg

   import core(exists_intro, exists_elim);

   sort T : Sort;

   lemma repack(P : T → Prop, ex := ∃ (x : T) st P(x))
     ⊢ ∃ (x : T) st P(x);
   proof
     by exists_elim(T, P, ∃ (x : T) st P(x)) cases
       case ⊢ ∃ (x : T) st P(x); proof by ex; qed;
       case x : T; witness := P(x) ⊢ ∃ (x : T) st P(x);
       proof
         by exists_intro(T, P, x) then ⊢ P(x); by witness;
       qed;
     qed;
   qed;

The second branch pulls out the witness ``x`` and the proof ``witness := P(x)``,
then feeds them straight back into ``exists_intro``. Trivial as a theorem, but it
shows the exact shape every real ``exists_elim`` proof has.

.. admonition:: Your turn
   :class: tip

   Combine the two moves: from ``∀ y. P(y)`` — ``P`` holds *everywhere* — produce a
   proof that ``∃ x. P(x)``.

   .. code-block:: alg

      import core(forall_elim, exists_intro);

      sort T : Sort;

      lemma somewhere(P : T → Prop, a : T, all := ∀ (y : T) st P(y))
        ⊢ ∃ (x : T) st P(x);
      proof
        by wip(?goal);
      wip;

   .. hint::

      Witness the existential at ``a`` first: ``by exists_intro(T, P, a)`` leaves
      ``then ⊢ P(a);``. Get ``P(a)`` by instantiating the universal —
      ``by forall_elim(T, P, a) then ⊢ ∀ (y : T) st P(y); by all;``.

If and only if: ``⇔``
=====================

A biconditional is just two implications bundled together, and its rules say
exactly that. ``biconditional_intro`` asks for both directions — ``A ⇒ B`` and
``B ⇒ A`` — so two premises, ``cases``, each closed by an assumed implication:

.. code-block:: alg

   import core(biconditional_intro);

   lemma equivalent(A B : Prop, fwd := A ⇒ B, bwd := B ⇒ A)
     ⊢ A ⇔ B;
   proof
     by biconditional_intro(A, B) cases
       case ⊢ A ⇒ B; proof by fwd; qed;
       case ⊢ B ⇒ A; proof by bwd; qed;
     qed;
   qed;

Going the other way, ``biconditional_elim_left`` extracts ``A ⇒ B`` from ``A ⇔ B``
(and ``biconditional_elim_right`` extracts ``B ⇒ A``) — one premise each, so
``then``. Between them you can take a ``⇔`` apart into whichever implication you
need, then finish with ``implication_elim`` from :doc:`logic`.

That's all of ``core``. Next we leave pure logic behind and start reasoning about
*data*.
