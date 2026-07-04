=======================
Rewriting with a motive
=======================

So far we've closed goals by matching an axiom's conclusion, and continued with
``then``. Now meet the workhorse of equational reasoning, ``rewrite_r``, which
takes an equation and rewrites *one chosen subterm* of the goal. Recall the rule
from :doc:`first-proofs`:

.. code-block:: alg

   rule rewrite_r(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(a)
     ────────────────────────
     ⊢ P(b)
   end;

The interesting argument is the last one, ``P`` — the **motive**. It's a function
``T → Prop``: a proposition with a *hole*, and the hole marks exactly where the
equation lands. Given ``eq : a = b``, ``rewrite_r`` swaps ``a`` for ``b`` at that
spot.

Writing motives as ``λ (x : T) st …`` gets old fast, so ``_`` is sugar for a
lambda: the motive ``n = _`` means ``λ (x : Nat) st n = x``. The ``_`` is the slot
the equation's sides plug into.

Let's re-prove ``zero_left_flip`` — this time by rewriting instead of flipping
with ``symmetry``:

.. code-block:: alg

   import nat;
   import core(refl, rewrite_r);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by rewrite_r(Nat, 0 + n, n, add_zero_left(n), n = _)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

Read the ``rewrite_r`` call as: with the equation ``0 + n = n`` (that's
``add_zero_left(n)``, so ``a = 0 + n`` and ``b = n``), and the motive ``n = _``,
rewrite ``0 + n`` to ``n``. Plug the two sides into the hole to see what it does:

- ``a`` in the hole → ``n = 0 + n`` — that's our current goal.
- ``b`` in the hole → ``n = n`` — the new goal after the rewrite.

So the step turns ``n = 0 + n`` into ``n = n``, which ``refl`` closes. The motive
is how you *point* at the ``0 + n`` on the right rather than the ``n`` on the left.

When the motive misses
======================

The motive has to reproduce the goal when the equation fills the hole. Aim it at
the wrong subterm and you'll get a very common error. Suppose we write ``_ = n``
by mistake:

.. code-block:: alg

   import nat;
   import core(refl, rewrite_r);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by rewrite_r(Nat, 0 + n, n, add_zero_left(n), _ = n)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

.. code-block:: text

   error: tactic `rewrite_r`: rule conclusion does not match the current goal

Here ``_ = n`` is ``λ (x : Nat) st x = n``. Filling the hole gives ``0 + n = n``
and ``n = n`` — and *neither* is the goal ``n = 0 + n``. The checker is telling you
that, with this motive, the rewrite step can't produce the goal you're standing on.
When you hit "rule conclusion does not match the current goal" on a ``rewrite_r``,
the motive is almost always the culprit: move the ``_`` to the subterm you actually
mean to rewrite.

No sugar, same proof
====================

``_`` is *only* shorthand. The motive is a plain lambda, and writing it out
long-hand checks identically:

.. code-block:: alg

   import nat;
   import core(refl, rewrite_r);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by rewrite_r(Nat, 0 + n, n, add_zero_left(n), λ (x : Nat) st n = x)
     then ⊢ n = n;
     by refl(Nat, n);
   qed;

Reach for ``_`` when the motive is obvious, and spell out the lambda when you want
to be explicit about the bound variable. ``rewrite_l`` is the mirror image — it
rewrites ``b`` to ``a`` — and the induction proof in :doc:`induction` puts
``rewrite_r`` to work with a hypothesis as its equation.
