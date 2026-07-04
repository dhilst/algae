=================
Your first proofs
=================

Axioms give operators meaning
=============================

An **axiom** asserts a sequent as true without proof. Operators are born
meaningless; equational axioms are how they earn their meaning. ``nat.alg`` gives
``+`` its personality with two of them:

.. code-block:: alg

   axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
   axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);

Now a plot twist that trips up newcomers. Algae's built-in notion of "the same
term" — **definitional equality** (``defeq``) — is **α/β-equivalence only**. Two
terms are equal when they share a beta normal form, full stop. Operators are
**inert constants**: the checker never *evaluates* them, so ``0 + 0`` does **not**
quietly collapse to ``0``. An axiom only takes effect where a proof explicitly
reaches for it.

The simplest way to reach for one is to close a goal that an axiom's conclusion
already matches. Instantiate ``add_zero_left`` at ``n = 0`` and its conclusion
``0 + n = n`` reads ``0 + 0 = 0`` — which is exactly our goal, so it closes on the
spot. ``import nat;`` brings ``0``, ``+``, and ``add_zero_left`` into scope:

.. code-block:: alg

   import nat;

   lemma zero_plus_zero
     ⊢ 0 + 0 = 0;
   proof
     by add_zero_left(0);
   qed;

There it is — your first proof. One line of ``by``, one ``qed``.

To apply an equation to a *subterm* — rewriting ``0 + 0`` to ``0`` *inside* a
bigger goal instead of matching the whole thing — you reach for the explicit
congruence rules ``rewrite_r`` / ``rewrite_l`` (coming up). There's no hidden
computation step anywhere: every use of an equation is a rule you can point to in
the proof.

The ``core`` module hands you ``refl``, which proves anything equal to itself:

.. code-block:: alg

   axiom refl(T : Sort, x : T)
     ⊢ x = x;

You instantiate it at the point of use — ``by refl(Nat, 0)`` proves ``0 = 0``.
But remember: ``defeq`` is α/β only, so ``refl`` closes ``a = b`` **only** when
``a`` and ``b`` are already α/β-equal. ``refl(Nat, 0)`` proves ``0 = 0`` but
**not** ``0 + 0 = 0``.

Rules: proofs that branch
=========================

An **inference rule** has premises above a line and a conclusion below it.
Applying a rule to a goal that matches its conclusion hands you one new subgoal
per premise. Here's ``symmetry`` from ``core``:

.. code-block:: alg

   rule symmetry(T : Sort, x y : T)
     ⊢ x = y
     ────────────────────────
     ⊢ y = x
   end;

To use it you say ``by symmetry(...)``. Its single premise leaves **one** goal
still to prove, so you continue *the same block* with ``then``: restate that goal,
then knock it out with the next ``by``. Watch us flip ``add_zero_left`` around —
proving ``n = 0 + n`` from ``0 + n = n``:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by symmetry(Nat, 0 + n, n)   # conclusion y = x matches goal n = 0 + n
     then ⊢ 0 + n = n;            # the one remaining subgoal
     by add_zero_left(n);         # discharged by the axiom
   qed;

The rhythm is always the same: **a step leaves subgoals; ``then`` continues a
single one, ``cases`` splits several.** An axiom (or any premise-free fact) leaves
*zero* subgoals, so it closes the goal outright — which is why
``by add_zero_left(n);`` ends the chain with no ``then``.

.. note::

   ``by symmetry(T, a, b)`` passes three *arguments*, matched against the rule's
   parameters ``(T, x, y)`` and typechecked like operator arguments. The current
   *goal* is not passed — it's matched against the rule's conclusion. A rule adds
   exactly one thing over an axiom: its premises become new subgoals.

   Parameters can be terms, sorts, predicates (``P : T → Prop``), or **proof
   arguments** — a parameter written ``eq := a = b`` wants a *proof reference*
   whose statement is ``a = b``, not a term. ``rewrite_r`` uses one:

   .. code-block:: alg

      rule rewrite_r(T : Sort, a b : T, eq := a = b, P : T → Prop)
        ⊢ P(a)
        ────────────────────────
        ⊢ P(b)
      end;

The shape of a proof
====================

A proof block is the keyword ``proof``, a chain of ``by`` steps, and a terminator
``qed`` (complete) or ``wip`` (still cooking). Every ``by`` step has exactly one
of three outcomes, and its shape follows the number of subgoals it leaves:

- **zero** — ``by refl(Nat, 0);`` closes the goal.
- **one** — ``by symmetry(...) then <goal>; by …`` continues the *same* block,
  no nesting; the ``then`` restates the single remaining subgoal.
- **many** — ``by induction(...) cases <case> <case> …`` branches, one ``case``
  per subgoal (each ``case`` has its own nested ``proof … qed``).

So a proof reads top to bottom: a straight ``by … then … by …`` chain for
single-goal steps, splitting into ``cases`` only where a rule genuinely branches.
``then`` may only follow a step that leaves one goal, ``cases`` a step that leaves
two or more, and a ``case`` is legal only inside a ``cases`` block.

And when you're not sure what to write next? Don't guess — leave a hole. That's
the whole next chapter.
