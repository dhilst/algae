========
Tutorial
========

Welcome! **Algae** is a small proof and algebraic-specification language. You hand
it a vocabulary (sorts and operators), a few facts (axioms and inference rules),
and then you *prove things* — by building explicit proof trees that a tiny, deeply
suspicious kernel re-checks step by step. No hand-waving survives.

Here's the good news: you don't have to memorise a wall of syntax before your
first proof. We'll write a one-liner almost immediately, and from there the
checker's **holes** feature (:doc:`holes`) does a lot of the teaching for you — it
literally prints the goal, what's in scope, and which tactics might work next.

.. tip::

   Every ``alg`` block on this site is a **live editor**. Edit it and press
   **Check ▶** to run the real kernel — compiled to WebAssembly — right in your
   browser. Break things on purpose; the kernel is unbribable and will point at
   exactly where you went wrong.

We work through the real standard library — ``core``, ``nat``, ``option``,
``group``, ``monad`` — so nothing here is throwaway toy code. It's all
``import``-able, and it all verifies.

.. toctree::
   :maxdepth: 2

   worlds
   first-proofs
   holes
   rewrite
   induction
   theories
