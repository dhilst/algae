===================
The logical toolkit
===================

Open |core.alg| and scroll past the equality rules — the middle of the file is a
little library of natural deduction: one pair of rules for each connective, one to
**build** it and one (or more) to **use** it. Learn to spot that pattern and the
whole module falls into place.

We'll state each fact about *abstract* propositions ``A``, ``B``, ``C : Prop`` — so
the logic stays in focus, with no equations to distract us. When a proof needs to
*start* from some assumption, we take it as a **lemma parameter**: writing
``x := A`` in a lemma's parameter list means "``x`` is a proof of ``A``," and you
discharge a goal that matches it with ``by x``. Keep that reading in mind — half the
tour is just plugging assumptions into holes.

Conjunction: ``∧``
==================

To *build* a conjunction you need both halves. ``and_intro`` takes a proof of ``A``
and a proof of ``B`` and hands back ``A ∧ B`` — two premises, so we branch with
``cases`` and close each with the matching assumption:

.. code-block:: alg

   import core(and_intro);

   lemma both(A B : Prop, x := A, y := B)
     ⊢ A ∧ B;
   proof
     by and_intro(A, B) cases
       case ⊢ A; proof by x; qed;
       case ⊢ B; proof by y; qed;
     qed;
   qed;

To *use* a conjunction, take it apart. ``and_left`` turns ``A ∧ B`` back into ``A``
(and ``and_right`` into ``B``). One premise — the conjunction — so we continue with
``then``, and discharge it with the ``both`` we were handed:

.. code-block:: alg

   import core(and_left);

   lemma just_left(A B : Prop, both := A ∧ B)
     ⊢ A;
   proof
     by and_left(A, B) then ⊢ A ∧ B; by both;
   qed;

Notice the rhythm: ``by and_left(A, B)`` says "I'm going to get ``A`` out of the
conjunction ``A ∧ B``," and the ``then`` goal is the whole conjunction you now owe a
proof of — which ``both`` supplies.

.. admonition:: Your turn
   :class: tip

   Conjunction doesn't care about order. Given a proof of ``A ∧ B``, prove ``B ∧ A``.

   .. code-block:: alg

      import core(and_intro, and_left, and_right);

      lemma and_comm(A B : Prop, both := A ∧ B)
        ⊢ B ∧ A;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``and_intro(B, A)`` splits the goal into ``⊢ B`` and ``⊢ A`` — two goals, so
      ``cases``. Get the ``B`` half with ``and_right(A, B)`` and the ``A`` half with
      ``and_left(A, B)``, each ``then``-ing on ``A ∧ B`` and closing ``by both``.

Disjunction: ``∨``
==================

Building a disjunction only needs *one* side. ``or_intro_left`` proves ``A ∨ B``
from ``A`` (and ``or_intro_right`` from ``B``) — a single premise, so ``then``:

.. code-block:: alg

   import core(or_intro_left);

   lemma pick_left(A B : Prop, x := A)
     ⊢ A ∨ B;
   proof
     by or_intro_left(A, B) then ⊢ A; by x;
   qed;

*Using* a disjunction is the interesting one, because you don't know which side
holds. ``or_elim`` makes you prove your goal **both ways** — once assuming ``A``,
once assuming ``B``. Three premises (the disjunction plus the two branches), so
three ``case`` s. Here's disjunction's own commutativity:

.. code-block:: alg

   import core(or_elim, or_intro_left, or_intro_right);

   lemma or_comm(A B : Prop, d := A ∨ B)
     ⊢ B ∨ A;
   proof
     by or_elim(A, B, B ∨ A) cases
       case ⊢ A ∨ B; proof by d; qed;
       case P := A ⊢ B ∨ A; proof by or_intro_right(B, A) then ⊢ A; by P; qed;
       case Q := B ⊢ B ∨ A; proof by or_intro_left(B, A) then ⊢ B; by Q; qed;
     qed;
   qed;

Two new things here. First, where do ``P`` and ``Q`` come from? **A rule that lets
you assume something names the assumption after its own premise.** In |core.alg|,
``or_elim`` 's branches are written ``P := P ⊢ R`` and ``Q := Q ⊢ R``, so inside the
left branch your new hypothesis is called ``P`` and in the right branch it's ``Q``.
You discharge it exactly like a lemma parameter — ``by P``. Second, the branches
*build* the flipped disjunction with the intro rules we just met.

.. admonition:: Your turn
   :class: tip

   Build the *right*-hand disjunct this time.

   .. code-block:: alg

      import core(or_intro_right);

      lemma pick_right(A B : Prop, y := B)
        ⊢ A ∨ B;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``or_intro_right(A, B)`` proves ``A ∨ B`` from ``⊢ B`` — the *right* side. One
      premise means ``then ⊢ B;``, and you already hold a proof of ``B``: ``by y``.

Implication: ``⇒``
==================

To *build* an implication ``A ⇒ B`` you assume ``A`` and prove ``B``.
``implication_intro`` introduces the antecedent as a hypothesis — named ``P``, after
its premise ``P := P ⊢ Q`` — and asks you to reach ``B``. The smallest example is
the identity ``A ⇒ A``, where the assumption *is* the goal:

.. code-block:: alg

   import core(implication_intro);

   lemma id(A : Prop)
     ⊢ A ⇒ A;
   proof
     by implication_intro(A, A) then P := A ⊢ A; by P;
   qed;

To *use* an implication, feed it its antecedent. ``implication_elim`` is plain modus
ponens: from ``A ⇒ B`` and ``A``, conclude ``B``. Two premises, ``cases``, both
discharged from assumptions we were handed:

.. code-block:: alg

   import core(implication_elim);

   lemma mp(A B : Prop, f := A ⇒ B, x := A)
     ⊢ B;
   proof
     by implication_elim(A, B) cases
       case ⊢ A ⇒ B; proof by f; qed;
       case ⊢ A; proof by x; qed;
     qed;
   qed;

Negation and falsehood
======================

Negation is really implication in disguise: ``¬A`` means "``A`` leads to
absurdity." So proving ``¬A`` from a proof that ``A ⇒ False`` is almost a
tautology — ``negation_intro`` assumes ``A`` (as ``P``), and we run the implication
to reach ``False``:

.. code-block:: alg

   import core(negation_intro, implication_elim);

   lemma neg_from_imp(A : Prop, f := A ⇒ False)
     ⊢ ¬A;
   proof
     by negation_intro(A) then P := A ⊢ False;
     by implication_elim(A, False) cases
       case ⊢ A ⇒ False; proof by f; qed;
       case ⊢ A; proof by P; qed;
     qed;
   qed;

And once you *have* ``False``, you have everything — ``false_elim`` proves any
proposition at all (the principle of explosion):

.. code-block:: alg

   import core(false_elim);

   lemma explosion(A : Prop, bad := False)
     ⊢ A;
   proof
     by false_elim(A) then ⊢ False; by bad;
   qed;

The rule that *produces* ``False`` is ``negation_elim``: from ``A`` and ``¬A`` — a
contradiction — it derives ``False`` (two premises, ``cases``). Chain it into
``false_elim`` and a contradiction proves anything at all.

.. admonition:: Your turn
   :class: tip

   Put the last two together: from a proof of ``A`` and a proof of ``¬A``, derive a
   completely unrelated ``C``.

   .. code-block:: alg

      import core(false_elim, negation_elim);

      lemma clash(A C : Prop, x := A, nx := ¬A)
        ⊢ C;
      proof
        by wip(?goal);
      wip;

   .. hint::

      Start with ``by false_elim(C) then ⊢ False;`` — now you only owe ``False``.
      Reach it with ``negation_elim(A)``, whose two ``cases`` are ``⊢ A`` (close
      ``by x``) and ``⊢ ¬A`` (close ``by nx``).

With the connectives in hand, the only thing left in ``core`` is the quantifiers —
and they're next.
