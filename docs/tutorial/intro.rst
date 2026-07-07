================
What is Algae?
================

**Algae** is a small language for two things that turn out to be the same thing:

- **Algebraic specification** — you describe a world by naming its *sorts* (base
  types) and *operators* (function symbols), then pin their meaning down with a
  handful of *axioms* and *inference rules*.
- **Proof** — once the world has a vocabulary and some facts, you *prove things
  about it* by building explicit proof trees that a tiny, deeply suspicious
  **kernel** re-checks step by step. No hand-waving survives.

That kernel is the whole point. It is small, it does not guess, and it is not
interested in your good intentions: a proof is correct only when every step lines
up exactly. When something is wrong it points at the spot and tells you why.

Two ways to run it
==================

Algae runs in two places, from the *same* kernel:

- **In your terminal**, via the ``algae-cli`` command-line tool — you write
  ``.alg`` files and run ``algae verify file.alg``. This is what you'd use to
  check a real development or wire Algae into a build.
- **In your browser**, via the kernel compiled to WebAssembly. Every Algae code
  block on this site is a *live editor* backed by that WebAssembly build — you
  edit it and check it on the spot, with nothing to install and no code leaving
  the page.

.. tip::

   **This tutorial uses the in-browser version.** Every ``alg`` block below is a
   real editor running the real kernel. Press **Check ▶** (or **Ctrl-Enter**) to
   run it; break things on purpose and watch the kernel object. The next chapter
   is a quick tour of that editor so you know which buttons to press.

Where we're headed
==================

We'll build up slowly and without assuming you've seen a proof assistant before:
first the editor, then just enough classical logic, then how Algae writes
inference rules and how proofs are assembled from them, and finally sorts,
specifications, and equational reasoning. From there the later chapters work
through the real standard library — ``core``, ``nat``, ``list``, ``group``,
``monad`` — so nothing here is throwaway toy code. It all ``import``\ s, and it
all verifies.
