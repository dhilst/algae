=======================
Quantifiers and ``⇔``
=======================

The last stretch of |core.alg| is the quantifiers, ``∀`` and ``∃``. They follow the
same *build one / use one* pattern as the connectives, with one twist: their rules
carry a **motive** — a predicate ``T → Prop`` you write with the ``_`` sugar from
:doc:`../induction`, marking the spot the bound variable goes.

For all: ``∀``
==============

To prove ``∀ x. P(x)``, prove ``P(x)`` for a *fresh, arbitrary* ``x``.
``forall_intro`` hands you that eigenvariable — one premise, so ``then``, and the
``then`` carries ``x`` into its context:

.. code-block:: alg

   import core(refl, forall_intro);

   sort T : Sort;

   lemma everything_is_itself
     ⊢ ∀ (x : T) st x = x;
   proof
     by forall_intro(T, _ = _)
     then x : T ⊢ x = x;
     by refl(T, x);
   qed;

The ``_ = _`` is the motive ``λ (x : T) st x = x`` — the property we're proving for
all ``x``. Because ``x`` was introduced fresh, proving ``x = x`` for it counts as
proving it for everyone (that's the eigenvariable freshness from :doc:`../induction`).

To *use* a ``∀``, instantiate it at a specific term. ``forall_elim`` takes
``∀ y. P(y)`` and, given a term ``x``, yields ``P(x)``:

.. code-block:: alg

   import core(forall_elim);

   sort T : Sort;
   axiom all_eq ⊢ ∀ (y : T) st y = y;

   lemma at_a_point(x : T)
     ⊢ x = x;
   proof
     by forall_elim(T, _ = _, x)
     then ⊢ ∀ (y : T) st y = y;
     by all_eq;
   qed;

The third argument, ``x``, is the point you're instantiating at; the ``then`` goal
is the universal statement you're drawing it from.

There exists: ``∃``
===================

Building a ``∃`` means producing a **witness**. ``exists_intro`` takes a term and a
proof that the property holds *of that term*:

.. code-block:: alg

   import core(refl, exists_intro);

   sort T : Sort;
   op a : → T;

   lemma there_is_one
     ⊢ ∃ (x : T) st x = x;
   proof
     by exists_intro(T, _ = _, a)
     then ⊢ a = a;
     by refl(T, a);
   qed;

We offered ``a`` as the witness, so the leftover goal is the property at ``a`` —
``a = a``.

*Using* a ``∃`` is the dual of using a ``∨``: you get a witness but you don't get to
know which one, so whatever you conclude must hold no matter who it is.
``exists_elim`` gives you a fresh ``x`` and the hypothesis that ``P(x)`` holds, and
asks you to reach your goal from there — two premises, ``cases``:

.. code-block:: alg

   import core(refl, exists_elim);

   sort T : Sort;
   op a : → T;
   axiom something ⊢ ∃ (x : T) st x = x;

   lemma conclude_a_a
     ⊢ a = a;
   proof
     by exists_elim(T, _ = _, a = a) cases
       case
         ⊢ ∃ (x : T) st x = x;
       proof
         by something;
       qed;
       case
         x : T;
         witness := x = x;
         ⊢ a = a;
       proof
         by refl(T, a);
       qed;
     qed;
   qed;

The second branch introduces both the witness ``x`` and the hypothesis
``witness := x = x``. Our goal ``a = a`` doesn't need them, but a real proof usually
would.

.. admonition:: Your turn
   :class: tip

   Prove a universal from scratch.

   .. code-block:: alg

      import core(refl, forall_intro);

      sort T : Sort;

      lemma all_reflexive
        ⊢ ∀ (x : T) st x = x;
      proof
        by wip(?goal);
      wip;

   .. hint::

      Start with ``by forall_intro(T, _ = _)`` — that's a single premise, so
      continue with ``then x : T ⊢ x = x;`` and finish with ``by refl(T, x);``.

If and only if: ``⇔``
=====================

A biconditional is just two implications bundled together, and its rules say
exactly that. ``biconditional_intro`` asks for both directions — ``P ⇒ Q`` and
``Q ⇒ P`` — so two premises, ``cases``:

.. code-block:: alg

   import core(biconditional_intro);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom forward  ⊢ (a = a) ⇒ (b = b);
   axiom backward ⊢ (b = b) ⇒ (a = a);

   lemma equivalent
     ⊢ (a = a) ⇔ (b = b);
   proof
     by biconditional_intro(a = a, b = b) cases
       case
         ⊢ (a = a) ⇒ (b = b);
       proof
         by forward;
       qed;
       case
         ⊢ (b = b) ⇒ (a = a);
       proof
         by backward;
       qed;
     qed;
   qed;

Going the other way, ``biconditional_elim_left`` extracts ``P ⇒ Q`` from
``P ⇔ Q`` (and ``biconditional_elim_right`` extracts ``Q ⇒ P``) — one premise each,
so ``then``. Between them you can take a ``⇔`` apart into whichever implication you
need, then finish with ``implication_elim`` from :doc:`logic`.

That's all of ``core``. Next we leave pure logic behind and start reasoning about
*data*.
