===================
The logical toolkit
===================

Open |core.alg| and scroll past the equality rules — the middle of the file is a
little library of natural deduction: one pair of rules for each connective, one to
**build** it and one (or more) to **use** it. Learn to spot that pattern and the
whole module falls into place.

We'll prove tiny facts about a two-element world so the logic stays in focus:

.. code-block:: alg

   sort T : Sort;
   op a : → T;
   op b : → T;

Conjunction: ``∧``
==================

To *build* a conjunction you need both halves. ``and_intro`` takes a proof of ``P``
and a proof of ``Q`` and hands back ``P ∧ Q`` — two premises, so we branch with
``cases``:

.. code-block:: alg

   import core(and_intro, refl);

   sort T : Sort;
   op a : → T;
   op b : → T;

   lemma both
     ⊢ a = a ∧ b = b;
   proof
     by and_intro(a = a, b = b) cases
       case
         ⊢ a = a;
       proof
         by refl(T, a);
       qed;
       case
         ⊢ b = b;
       proof
         by refl(T, b);
       qed;
     qed;
   qed;

To *use* a conjunction, take it apart. ``and_left`` turns ``P ∧ Q`` back into
``P`` (and ``and_right`` into ``Q``). One premise — the conjunction — so we
continue with ``then``:

.. code-block:: alg

   import core(and_left);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom both ⊢ a = a ∧ b = b;

   lemma just_the_left
     ⊢ a = a;
   proof
     by and_left(a = a, b = b)
     then ⊢ a = a ∧ b = b;
     by both;
   qed;

Notice the rhythm: ``by and_left(a = a, b = b)`` says "I'm going to get ``a = a``
out of the conjunction ``a = a ∧ b = b``," and the ``then`` goal is the whole
conjunction you now owe a proof of.

.. admonition:: Your turn
   :class: tip

   Prove the same conjunction, but flipped. Replace the hole with a real proof and
   press **Check ▶**.

   .. code-block:: alg

      import core(and_intro, refl);

      sort T : Sort;
      op a : → T;
      op b : → T;

      lemma flipped
        ⊢ b = b ∧ a = a;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``and_intro`` splits ``⊢ P ∧ Q`` into ``⊢ P`` and ``⊢ Q`` — two goals, so use
      ``cases``. Here both halves are reflexive equations; close each with
      ``refl(T, b)`` and ``refl(T, a)``.

Disjunction: ``∨``
==================

Building a disjunction only needs *one* side. ``or_intro_left`` proves ``P ∨ Q``
from ``P`` (and ``or_intro_right`` from ``Q``) — a single premise, so ``then``:

.. code-block:: alg

   import core(or_intro_left, refl);

   sort T : Sort;
   op a : → T;
   op b : → T;

   lemma pick_a_side
     ⊢ a = a ∨ b = b;
   proof
     by or_intro_left(a = a, b = b)
     then ⊢ a = a;
     by refl(T, a);
   qed;

*Using* a disjunction is the interesting one, because you don't know which side
holds. ``or_elim`` makes you prove your goal **both ways** — once assuming ``P``,
once assuming ``Q``. Three premises (the disjunction plus the two cases), so three
``case`` branches:

.. code-block:: alg

   import core(or_elim, refl);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom either ⊢ a = b ∨ b = a;

   lemma reach_a_a
     ⊢ a = a;
   proof
     by or_elim(a = b, b = a, a = a) cases
       case
         ⊢ a = b ∨ b = a;
       proof
         by either;
       qed;
       case
         h := a = b;
         ⊢ a = a;
       proof
         by refl(T, a);
       qed;
       case
         h := b = a;
         ⊢ a = a;
       proof
         by refl(T, a);
       qed;
     qed;
   qed;

Each branch introduces a hypothesis (``h := a = b``) you may lean on — here we
don't need it, since ``a = a`` is true regardless. That's the shape of every
elimination: assume, then discharge.

.. admonition:: Your turn
   :class: tip

   Build the *right*-hand disjunct this time.

   .. code-block:: alg

      import core(or_intro_right, refl);

      sort T : Sort;
      op a : → T;
      op b : → T;

      lemma other_side
        ⊢ a = b ∨ b = b;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``or_intro_right(P, Q)`` proves ``P ∨ Q`` from ``⊢ Q`` — the *right* side. So
      here you only owe ``⊢ b = b``. One premise means ``then``, then ``refl(T, b)``.

Implication: ``⇒``
==================

To *build* an implication ``P ⇒ Q`` you assume ``P`` and prove ``Q``.
``implication_intro`` introduces the antecedent as a hypothesis — one premise, so
``then`` (and the ``then`` carries the new hypothesis in its context):

.. code-block:: alg

   import core(implication_intro, refl);

   sort T : Sort;
   op a : → T;
   op b : → T;

   lemma weaken
     ⊢ (b = b) ⇒ (a = a);
   proof
     by implication_intro(b = b, a = a)
     then h := b = b ⊢ a = a;
     by refl(T, a);
   qed;

To *use* an implication, feed it its antecedent. ``implication_elim`` is plain
modus ponens: from ``P ⇒ Q`` and ``P``, conclude ``Q``. Two premises, ``cases``:

.. code-block:: alg

   import core(implication_elim);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom rule_ab ⊢ (a = a) ⇒ (b = b);
   axiom have_a  ⊢ a = a;

   lemma modus_ponens
     ⊢ b = b;
   proof
     by implication_elim(a = a, b = b) cases
       case
         ⊢ (a = a) ⇒ (b = b);
       proof
         by rule_ab;
       qed;
       case
         ⊢ a = a;
       proof
         by have_a;
       qed;
     qed;
   qed;

Negation and falsehood
======================

Negation is really implication in disguise: ``¬P`` means "``P`` leads to
absurdity." ``negation_intro`` asks you to assume ``P`` and derive ``False``; here
we lean on an (intentionally inconsistent) axiom so we can watch the mechanics:

.. code-block:: alg

   import core(negation_intro);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom absurd ⊢ False;

   lemma not_ab
     ⊢ ¬(a = b);
   proof
     by negation_intro(a = b)
     then h := a = b ⊢ False;
     by absurd;
   qed;

And once you *have* ``False``, you have everything — ``false_elim`` proves any
proposition at all (the principle of explosion):

.. code-block:: alg

   import core(false_elim);

   sort T : Sort;
   op a : → T;
   op b : → T;
   axiom absurd ⊢ False;

   lemma anything
     ⊢ a = b;
   proof
     by false_elim(a = b)
     then ⊢ False;
     by absurd;
   qed;

The missing partner, ``negation_elim``, closes the loop: from ``P`` and ``¬P`` it
derives ``False`` (two premises, ``cases``) — the collision that ``false_elim``
then turns into anything.

.. admonition:: Your turn
   :class: tip

   Explosion in action: given ``False``, prove a completely unrelated equation.

   .. code-block:: alg

      import core(false_elim);

      sort T : Sort;
      op a : → T;
      op b : → T;
      axiom boom ⊢ False;

      lemma out_of_nothing
        ⊢ b = a;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``false_elim(P)`` proves ``⊢ P`` from a single premise ``⊢ False``. Point the
      goal at ``b = a``, continue with ``then ⊢ False;``, and discharge it with
      ``by boom;``.

With the connectives in hand, the only thing left in ``core`` is the quantifiers —
and they're next.
