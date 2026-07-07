================
Building proofs
================

One rule application is a single step. A *proof* is many of them fitted
together, and it helps to picture how.

Proofs are dominoes
===================

Think of the rules as pieces in a game of dominoes. Each piece has a top (its
conclusion) and a bottom (its premises), and you can lay one piece against
another only where they line up: a premise you still owe on one piece is
answered by the conclusion of the next.

You start by laying down a piece whose top matches your **goal**. That piece may
leave one open end (a single premise) that you extend with another piece, or it
may **branch** into several open ends (several premises) — one for each subgoal.
You keep laying pieces down each open end until every end is capped. And you cap
an end with an **axiom** or an **assumption** — a piece with no bottom, nothing
left to answer. When every branch is capped, the chain is complete and the proof
closes.

That's why the number of premises drives everything: a one-premise rule extends
a single line (``then``), a many-premise rule forks it (``cases``), and a
zero-premise fact caps the line it's on.

Because each step hangs its subgoals *beneath* it and every branch ends in a cap,
the finished structure is a **tree**: the goal at the root, rule applications for
nodes, and axioms or assumptions for leaves. This is the **proof tree**, and it's
the thing the kernel actually checks. A proof written in Algae is just that tree,
laid out in text.

A branching proof, end to end
=============================

Let's prove that conjunction doesn't care about order: from a proof of ``A ∧ B``
we build ``B ∧ A``. This uses all three conjunction rules at once — a nice
domino chain that forks and then caps each branch. We spell the rules out in the
buffer instead of ``import``-ing them, so you can see exactly what each ``by``
step is matching against:

.. code-block:: alg

   rule and_intro(P Q : Prop)
     ⊢ P;
     ⊢ Q
     ────────────────────────
     ⊢ P ∧ Q
   end;

   rule and_left(P Q : Prop)
     ⊢ P ∧ Q
     ────────────────────────
     ⊢ P
   end;

   rule and_right(P Q : Prop)
     ⊢ P ∧ Q
     ────────────────────────
     ⊢ Q
   end;

   lemma and_comm(A B : Prop, both := A ∧ B)
     ⊢ B ∧ A;
   proof
     by and_intro(B, A) cases
       case ⊢ B; by and_right(A, B) then ⊢ A ∧ B; by both; qed;
       case ⊢ A; by and_left(A, B)  then ⊢ A ∧ B; by both; qed;
     qed;
   qed;

Follow the dominoes:

- The goal is ``⊢ B ∧ A``. Lay down ``and_intro`` — its conclusion ``P ∧ Q``
  matches with ``P = B``, ``Q = A``, so ``by and_intro(B, A)``. It has two
  premises, so the line **forks** into ``⊢ B`` and ``⊢ A`` — hence ``cases``.
- **Left branch, ``⊢ B``.** Lay ``and_right`` (conclusion ``⊢ Q``, matched with
  ``Q = B``). Its single premise is ``⊢ A ∧ B``, so we extend with
  ``then ⊢ A ∧ B;`` — and cap that end with our assumption, ``by both``.
- **Right branch, ``⊢ A``.** Symmetric: ``and_left`` (conclusion ``⊢ P``, with
  ``P = A``) leaves ``⊢ A ∧ B``, capped again ``by both``.

Both branches capped, both ``cases`` closed, ``qed``. Notice how ``both := A ∧ B``
— the whole conjunction we were handed — caps *both* branches; it's evidence we
can cite as many times as we like.

Draw the same proof as a tree and its shape jumps out — the goal at the root,
each ``by`` a node that hangs its subgoals below, and every leaf an assumption
that caps its branch:

.. code-block:: text

   ⊢ B ∧ A                              (the goal — root of the tree)
   └─ by and_intro(B, A)                two premises → the tree forks
      ├─ ⊢ B
      │  └─ by and_right(A, B)          one premise
      │     └─ ⊢ A ∧ B
      │        └─ by both               leaf: assumption caps the branch
      └─ ⊢ A
         └─ by and_left(A, B)           one premise
            └─ ⊢ A ∧ B
               └─ by both               leaf: assumption caps the branch

Read top to bottom it's the proof you wrote; read bottom to top it's the logical
argument — every branch climbs from the goal up to a fact you already hold. That
tree *is* the proof; the ``proof … qed`` text is just one way of writing it down.

What the checker actually checks
================================

The kernel (``algae-kernel``) is deliberately dim: it does not search for
proofs, and it will not fill anything in for you. For every ``by`` step it
performs exactly two checks:

1. **The rule fits the goal.** It instantiates the rule's *conclusion* with your
   arguments and confirms it matches the **current goal**. (``and_intro(B, A)``
   has conclusion ``B ∧ A`` — that had better be the goal.)
2. **You accounted for the premises.** It instantiates the rule's *premises* the
   same way and confirms the subgoals you continue with — your ``then`` goal, or
   your ``case`` goals — are exactly those premises. (``and_intro(B, A)`` demands
   premises ``⊢ B`` and ``⊢ A``; your two cases must be precisely those.)

Get either wrong and the step is rejected, pointing at the mismatch. There is no
third, hidden step where the checker "runs" anything or guesses — a proof is
correct only when every domino lines up under those two checks, all the way down.

.. admonition:: Everything is explicit — on purpose
   :class: note

   Unlike Lean 4 or Rocq (Coq), Algae keeps *all* the proof information in the
   syntax, and that was a deliberate design decision. Rocq, for example, drives
   proofs with a macro/tactic language that *generates* proof terms — which
   enables proof search, automation, and a great deal of power. That power comes
   at two costs: it is harder to learn, and you cannot read the proof off the
   source. (Rocq *can* print the underlying term, but it was never meant to be
   read.) Algae is built on two principles instead: **be simple**, and **be
   explicit** — the context, the goal, and the rule being applied are all
   deliberately required in the syntax, right where you can see them.

.. admonition:: Why so strict?
   :class: note

   Because a checker that guesses is a checker you can't trust. Every gap is one
   you must close explicitly, which is exactly what makes a passing proof mean
   something. When you *don't* know the next piece, that's what **holes** are
   for — drop a ``by wip(?goal)``, check, and let the kernel tell you the open
   end it's still waiting on.

.. admonition:: Your turn
   :class: tip

   Disjunction has the mirror shape: ``or_intro_left`` proves ``A ∨ B`` from just
   ``A`` — a *single* premise, so ``then``. Given a proof of ``A``, prove
   ``A ∨ B``. The buffer inspects the step with a trailing ``?``; **Check ▶** to
   see what it leaves.

   .. code-block:: alg

      rule or_intro_left(P Q : Prop)
        ⊢ P
        ────────────────────────
        ⊢ P ∨ Q
      end;

      lemma weaken(A B : Prop, x := A)
        ⊢ A ∨ B;
      proof
        by or_intro_left(A, B)?;
      wip;

   .. hint::

      ``or_intro_left(A, B)`` matches ``⊢ A ∨ B`` and leaves the one premise
      ``⊢ A``. Drop the ``?``, continue ``then ⊢ A;``, close ``by x;``, and make
      the terminator ``qed``.
