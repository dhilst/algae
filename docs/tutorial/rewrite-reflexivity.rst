===============================
Reflexivity and rewriting
===============================

Equations are the heart of an algebraic specification, so Algae gives you three
tools for them, all in ``core``: **reflexivity** to close ``x = x``, and the two
**rewrite** rules to apply an equation to *part* of a goal.

Reflexivity
===========

``refl`` proves anything equal to itself. It's an axiom — zero premises — so it
closes a goal outright:

.. code-block:: alg

   axiom refl(
     T : Sort,
     x : T
   )
     ⊢ x = x;

You instantiate it at the point of use: ``by refl(Nat, 0)`` closes ``0 = 0``.

.. code-block:: alg

   import nat;
   import core(refl);

   lemma zero_is_zero
     ⊢ 0 = 0;
   proof
     by refl(Nat, 0);
   qed;

But remember from :doc:`specs` that definitional equality is **α/β only**.
``refl`` closes ``a = b`` only when ``a`` and ``b`` are *already* α/β-equal, so
``refl(Nat, 0)`` proves ``0 = 0`` but **not** ``0 + 0 = 0`` — the operator ``+``
never evaluates. To bridge that gap you need an equation and a way to *apply* it.

Rewriting with a motive
=======================

The workhorses of equational reasoning are the two congruence rules. Each takes
an equation and rewrites **one chosen subterm** of the goal:

.. code-block:: alg

   rule forward(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(b)
     ────────────────────────
     ⊢ P(a)
   end;

   rule backward(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(a)
     ────────────────────────
     ⊢ P(b)
   end;

The interesting argument is the last one, ``P`` — the **motive**. It's a function
``T → Prop``: a proposition with a *hole*, and the hole marks exactly where the
equation lands. The names say which way the equation runs *at proof time*:

- **``forward``** takes ``eq : a = b`` and replaces ``a`` with ``b`` in the goal
  — following the equation the way you read it, left to right.
- **``backward``** takes ``eq : a = b`` and replaces ``b`` with ``a`` — using the
  equation right to left, against its natural reading.

The ``_`` sugar
===============

Writing motives as ``λ (x : T) st …`` gets old fast, so ``_`` is sugar for that
lambda: the motive ``n = _`` means ``λ (x : Nat) st n = x``. The ``_`` is the
slot the equation's sides plug into.

Let's prove ``n = 0 + n``. We have ``add_zero_left(n) : 0 + n = n``, and we want
to turn the ``0 + n`` on the right into ``n`` — replacing ``a`` (``0 + n``) with
``b`` (``n``), which is **``forward``**:

.. code-block:: alg

   import nat;
   import core(refl, forward);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by forward(Nat, 0 + n, n, add_zero_left(n), n = _)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

Read the call as: with the equation ``0 + n = n`` (so ``a = 0 + n`` and
``b = n``) and the motive ``n = _``, rewrite ``0 + n`` to ``n``. Plug each side
into the hole to see what it does:

- ``a`` in the hole → ``n = 0 + n`` — our current goal.
- ``b`` in the hole → ``n = n`` — the new goal after the rewrite.

So the step turns ``n = 0 + n`` into ``n = n``, which ``refl`` closes. The motive
is how you *point* at the ``0 + n`` on the right rather than the ``n`` on the
left.

``_`` is only shorthand — the motive is a plain lambda, and spelling it out
long-hand checks identically. These two lines are the same step:

.. code-block:: alg

   import nat;
   import core(refl, forward);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by forward(Nat, 0 + n, n, add_zero_left(n), λ (x : Nat) st n = x)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

Reach for ``_`` when the motive is obvious, and write the lambda when you want to
be explicit about the bound variable.

When the motive misses
======================

The motive has to reproduce the goal when the equation's ``a`` side fills the
hole. Aim it at the wrong subterm and you get a very common error. Suppose we
write ``_ = n`` by mistake:

.. code-block:: alg

   import nat;
   import core(refl, forward);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by forward(Nat, 0 + n, n, add_zero_left(n), _ = n)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

.. code-block:: text

   error: tactic `forward`: rule conclusion does not match the current goal
     the rule concludes:  (λ (x : Nat) st x = n)(0 + n)
     but the goal is:     n = 0 + n

Here ``_ = n`` is ``λ (x : Nat) st x = n``. With ``a = 0 + n`` in the hole it
produces ``0 + n = n`` — *not* the goal ``n = 0 + n``. When you hit "rule
conclusion does not match the current goal" on a rewrite, the motive is almost
always the culprit: move the ``_`` to the subterm you actually mean to touch.

The other direction: ``backward``
=================================

Use ``backward`` when you need to go the other way — expand a term to match an
equation's *right*-hand side. This is the everyday move in an induction step.
Here we're given the induction hypothesis ``ih : k + 0 = k`` and must prove
``s(k) + 0 = s(k)``; we rewrite the ``k`` inside ``s(_)`` *backward* into
``k + 0`` (replacing ``b = k`` with ``a = k + 0``) so that ``add_succ_left`` can
finish it:

.. code-block:: alg

   import nat;
   import core(backward);

   lemma succ_step(k : Nat, ih := k + 0 = k)
     ⊢ s(k) + 0 = s(k);
   proof
     by backward(Nat, k + 0, k, ih, s(k) + 0 = s(_))
     then ⊢ s(k) + 0 = s(k + 0);
     by add_succ_left(k, 0);
   qed;

Notice the equation here is ``ih`` — a *hypothesis*, not an axiom. Any proof of an
equality will do as the ``eq`` argument, which is what makes rewriting with your
induction hypothesis possible. You'll see this exact step inside a real induction
in :doc:`induction`.

.. admonition:: forward or backward?
   :class: note

   Pick by what you're doing to the goal, reading the equation as ``a = b``:
   to turn an ``a`` you can see into ``b``, use ``forward``; to turn a ``b`` into
   an ``a`` (usually to set up another rule), use ``backward``. If a rewrite is
   rejected with "rule conclusion does not match," you've either aimed the motive
   wrong or picked the wrong direction — try its mirror.
