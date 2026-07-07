========
Tutorial
========

Welcome! **Algae** is a small proof and algebraic-specification language. You hand
it a vocabulary (sorts and operators), a few facts (axioms and inference rules),
and then you *prove things* — by building explicit proof trees that a tiny, deeply
suspicious kernel re-checks step by step. No hand-waving survives.

This tutorial runs entirely **in your browser**. Every ``alg`` block below is a
live editor backed by the real kernel compiled to WebAssembly — edit it, press
**Check ▶**, and watch the kernel object. We start from the ground up: the
editor, then just enough logic, then how Algae writes rules and assembles proofs,
and finally sorts, specifications, and equational reasoning. The later chapters
work through the real standard library — ``core``, ``nat``, ``list``, ``group``,
``monad`` — so nothing here is throwaway toy code. It all ``import``\ s, and it
all verifies.

.. tip::

   You don't have to memorise a wall of syntax before your first proof. Break
   things on purpose; the kernel is unbribable and will point at exactly where
   you went wrong. And when you're not sure what to write next, leave a
   **hole** — the editor will tell you the goal and even suggest the fix.

.. toctree::
   :maxdepth: 2

   intro
   editor
   propositional-logic
   inference-rules
   backward-reasoning
   proofs
   specs
   rewrite-reflexivity
   proof-techniques
   induction
   errors
   theories
   tour/index
   auxiliary-lemmas
