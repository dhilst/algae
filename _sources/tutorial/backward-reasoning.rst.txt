==================
Reasoning backward
==================

Here's the twist that makes proofs click. You *read* an inference rule
top-to-bottom — premises, then conclusion. But you *prove* with it
**bottom-to-top**. You start from the goal you want and work backward toward
things you already have.

The mechanic
============

During a proof you're always staring at a **current goal**. To make progress:

1. **Find a rule whose conclusion matches your goal.** The rule's conclusion is
   a template; it matches if your goal has that shape.
2. **Apply it.** The goal is *replaced* by the rule's premises — because the
   rule promises that establishing the premises establishes the conclusion, and
   the conclusion is your goal.

How many goals you're left with is decided entirely by the **number of
premises**:

- **one premise** → the goal is replaced by that single new subgoal; you
  continue in the same line with ``then``.
- **more than one premise** → you get **one subgoal per premise**; you split
  into branches with ``cases``, one ``case`` each.
- **zero premises** → there's nothing left to establish, so the goal
  **closes** on the spot. A rule with no premises is an **axiom**, so *applying
  an axiom (or any premise-free fact) finishes a goal outright.*

That's the whole engine. A proof is just this step, repeated, until every branch
has closed.

Your first proof
================

Let's prove a conjunction. Suppose we're *handed* a proof of ``A`` and a proof of
``B``; we'll assemble ``A ∧ B``. In Algae you take assumptions as **lemma
parameters**: writing ``x := A`` in the parameter list means "``x`` is a proof of
``A``," and you discharge a goal that matches it by citing ``by x``.

.. code-block:: alg

   import core(and_intro);

   lemma both(A B : Prop, x := A, y := B)
     ⊢ A ∧ B;
   proof
     by and_intro(A, B) cases
       case ⊢ A; by x; qed;
       case ⊢ B; by y; qed;
     qed;
   qed;

Press **Check ▶**. Now let's read it as backward reasoning:

- The goal is ``⊢ A ∧ B``. We look for a rule whose conclusion is a conjunction —
  that's ``and_intro``, whose conclusion is ``⊢ P ∧ Q``. Matching against our
  goal sets ``P = A`` and ``Q = B``, which is why we call ``by and_intro(A, B)``.
- ``and_intro`` has **two** premises (``⊢ P`` and ``⊢ Q``), so applying it leaves
  **two** subgoals: ``⊢ A`` and ``⊢ B``. Two goals means we branch — ``cases`` —
  with one ``case`` per subgoal.
- The first ``case ⊢ A;`` is discharged ``by x`` — our assumption ``x := A`` is
  exactly a proof of ``A``, a premise-free fact, so it **closes** the goal. Its
  little block ends with ``qed``. Likewise ``⊢ B`` closes ``by y``.
- With both branches closed, the outer ``cases`` is done, and the proof closes
  with ``qed``.

The vocabulary
==============

Four keywords carry every proof:

- **``by <rule>(args)``** applies a rule (or axiom, or an assumption). The
  arguments fill the rule's parameters; the *goal itself is never passed* — it's
  matched against the rule's conclusion.
- **``then <goal>;``** continues after a step that left **one** subgoal. You
  restate that subgoal and keep going in the same block — no nesting.
- **``cases … case … case …``** branches after a step that left **several**
  subgoals: one ``case`` per branch, each its own little ``proof … qed``.
- **``qed``** closes a finished block; **``wip``** (work-in-progress) closes a
  block you haven't finished — the proof still checks, but as *in progress*.

.. admonition:: then vs. cases, precisely
   :class: note

   ``then`` may only follow a step that leaves **one** goal; ``cases`` a step
   that leaves **two or more**; and a premise-free step (an axiom or assumption)
   leaves **zero**, so it takes neither — it just ends with ``;``. Match the
   continuation to the number of premises and the proof writes itself.

.. admonition:: Your turn
   :class: tip

   ``and_left`` has a *single* premise (the conjunction), so it continues with
   ``then``. Given a proof ``h`` of ``A ∧ B``, pull out the left half. Fill the
   hole — press **Check ▶**, and if you're stuck, click the hole for a
   suggestion.

   .. code-block:: alg

      import core(and_left);

      lemma left_half(A B : Prop, h := A ∧ B)
        ⊢ A;
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``by and_left(A, B)`` matches the goal ``⊢ A`` (the rule's conclusion is
      ``⊢ P`` with ``P = A``), and leaves its one premise ``⊢ A ∧ B`` — continue
      ``then ⊢ A ∧ B;`` and close it ``by h;``.
