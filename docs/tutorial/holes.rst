==================
Proving with holes
==================

Here's the secret that makes Algae pleasant to *write* proofs in, not just read
them: you don't have to know the whole proof before you start. You leave a
**hole**, and the checker tells you what goes there.

``by wip;`` **admits** the current goal without proving it (a block that admits
must be closed with ``wip`` instead of ``qed``). Its more helpful cousin,
``by wip(?name)``, admits the goal *and* prints a **hole report**: the goal, the
context in scope, and candidate tactics. Grow your proof one step at a time.

.. tip::

   The reports below appear on the command line **and** inline in the editors on
   this page. Press **Check ▶** on the holed examples to watch it happen.

Start with just a skeleton and a hole:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by wip(?goal);
   wip;

Check it, and the kernel tells you where you are and where to look next:

.. code-block:: text

   found hole ?goal : proof

   Expected:
     n = 0 + n

   Context:
     n : Nat

   Goal:
     n = 0 + n

   Candidates:
     symmetry (rule)
     transitivity (rule)

``symmetry`` turns ``n = 0 + n`` into ``0 + n = n``. Apply it and slide the hole
into the ``then`` continuation to see what's left:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by symmetry(Nat, 0 + n, n)
     then ⊢ 0 + n = n;
     by wip(?rest);
   wip;

Now the hole reports ``0 + n = n``, with ``add_zero_left (fact)`` right there in
the candidates — exactly the axiom that closes it. Drop it in, swap the final
``wip`` for ``qed``, and you're done:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by symmetry(Nat, 0 + n, n)
     then ⊢ 0 + n = n;
     by add_zero_left(n);
   qed;

A module with any ``wip`` — holed or not — is **incomplete**: the checker reports
it and the run fails, so a hole can never sneak past as a finished proof.
Candidates are a best-effort hint (local hypotheses, facts and rules whose
conclusion matches the goal, and ``refl`` for a reflexive equation), not a
promise — but they're usually enough to find the next ``by``.

Holes inside a tactic
=====================

Once you've *picked* a tactic, a ``?`` helps you fill it in. Put ``?`` after a
whole application to **inspect** it — the checker applies the tactic and hands you
the next step, ready to paste:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by symmetry(Nat, 0 + n, n)?;
   wip;

.. code-block:: text

   Applying it leaves:
     ⊢ 0 + n = n

   Continue with:
     then ⊢ 0 + n = n;
     by wip?;

Or leave individual arguments as **named holes** ``?a`` and let the checker solve
them straight from the goal. ``symmetry``'s conclusion ``y = x`` must match
``n = 0 + n``, which pins down ``?a`` and ``?b`` — and even the sort ``?T``,
recovered by type inference:

.. code-block:: alg

   import nat;
   import core(symmetry);

   lemma zero_left_flip(n : Nat)
     ⊢ n = 0 + n;
   proof
     by symmetry(Nat, ?a, ?b) then ?g;
   wip;

.. code-block:: text

   Holes:
     ?a : Nat = 0 + n
     ?b : Nat = n

   Subgoal(s):
     ?g : ⊢ 0 + n = n

``by symmetry?;`` (no arguments at all) holes *every* parameter at once. Holes
even work in proof-argument positions: ``by rewrite_r(Nat, k + 0, k, ?eq, _)?;``
reports ``?eq : ⊢ k + 0 = k`` — the equation you still owe a proof of. And a hole
the goal *doesn't* pin down — a genuinely free choice, like ``transitivity``'s
middle term — is shown with its type and no value, so you know it's yours to pick.
