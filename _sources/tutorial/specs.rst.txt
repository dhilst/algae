========================
Algebraic specification
========================

So far the propositions were abstract — bare ``A``, ``B : Prop``. Now let's give
ourselves something concrete to talk *about*. This is the other half of Algae: it
is not only a proof language but an **algebraic specification** language, and this
chapter is about what that means.

Types from types
================

Before the vocabulary, four ways Algae builds a type out of simpler ones:

- ``A * B`` — a **pair**: one value of type ``A`` together with one of type
  ``B``.
- ``A | B`` — a **sum**: a value that is *either* an ``A`` *or* a ``B``.
- ``A → B`` — a **function** that takes an ``A`` and returns a ``B``.
- ``A * B → C`` — a **function of two arguments**: it takes an ``A`` *and* a
  ``B``, and returns a ``C``.

That last one matters because Algae has **no currying**. A function states exactly
how many arguments it takes: ``A * B → C`` is a genuine two-argument function, not
a one-argument function that returns another function (``A → (B → C)``). When you
apply it you supply *both* arguments at once — ``f(a, b)``.

Declare, don't define
=====================

The idea behind algebraic specification is this: instead of describing your code
with a programming language (or pseudocode), you **declare** the *types* — here
called **sorts** — and the *operations* that act on them. Note the word: declare,
not *define*. You name the sorts and operations **abstractly**. You say nothing
about what the values of a sort actually *are*, or how an operation is
*implemented* — you just declare that they exist and what their shapes are.

Then you give them meaning by specifying how the operations **relate to each
other**, and you do that with **equations**.

In the idiom of set theory, a tiny stack specification would read something like:

.. epigraph::

   Let ``S`` be the set of stacks and ``E`` the set of elements.
   Let ``push : E * S → S`` be the operation taking an element and a stack and
   returning the stack with that element on top.
   Let ``top : S → E`` return the top element.
   Then for every ``s : S`` and ``e : E`` the following must hold:

   .. code-block:: text

      top(push(e, s)) = e

Algae, as a specification language, is meant to express *exactly this* — but
instead of prose in a ``.txt`` file you write it in a ``.alg`` file, and the
checker can parse, typecheck, and verify it for you:

.. code-block:: alg

   sort Stack : Sort → Sort;   # stacks of some element type

   op empty : forall (A : Sort) st → Stack(A);      # the empty stack
   op push  : forall (A : Sort) st A * Stack(A) → Stack(A);
   op top   : forall (A : Sort) st Stack(A) → A;

   axiom top_push(A : Sort, x : A, s : Stack(A))
     ⊢ top(A, push(A, x, s)) = x;

.. admonition:: Constructors *are* operations
   :class: note

   You might expect a distinction between *constructors* (which build values) and
   *operations* (which consume them). Algae makes none. ``op empty : forall (A : Sort) st → Stack(A)``
   looks like a constructor and ``op push : forall (A : Sort) st A * Stack(A) → Stack(A)`` like an
   operation, but to Algae they are both just operators — symbols with a
   signature. There is no privileged set of "the real values"; there are only the
   operators and the equations that relate them.

Why declare and not define? Because at specification time you don't want to be
distracted by implementation. *Which data structure should back the stack — a
linked list? an array?* Not yet your concern; you're pinning down the *behaviour*,
rigorously and abstractly. That means less to write and less to read.

And the result is a real artifact. You can commit that ``.alg`` file to a
repository as the *true specification* of a module in your codebase — hand it to
another engineer, feed it to an LLM, or implement it yourself. Problems can still
surface during implementation (there's no silver bullet), but an unambiguous,
machine-checked source of truth for *what your code must do* has real advantages:
you can fix the spec the way you fix code, and there's no informal prose left to
hide ambiguity in.

.. admonition:: This stack is incomplete
   :class: warning

   Look again at what the spec above says — and doesn't. It never mentions
   ``top(A, empty(A))``: what is the top of an *empty* stack? Nothing here pins it down.
   We'll return to this exact gap in :doc:`errors` and see how to close it.

Sorts and operations, precisely
===============================

To restate the vocabulary now that you've seen it in use. A **sort** is a base
type; ``Stack : Sort → Sort`` is a sort *constructor* — ``Stack(A)`` is a sort for
each element sort ``A``. An **operation** (``op``) is a function symbol with a
signature, built from the type formers above — ``→`` for the result, ``*`` for a
tuple of arguments:

.. code-block:: alg

   op empty : forall (A : Sort) st → Stack(A);            # no arguments — just a result
   op push  : forall (A : Sort) st A * Stack(A) → Stack(A);  # two arguments: an A and a Stack(A)
   op pop   : forall (A : Sort) st Stack(A) → Stack(A);   # one argument
   op top   : forall (A : Sort) st Stack(A) → A;

That's a *vocabulary* and nothing more. ``empty``, ``push``, ``pop``, and ``top``
are just symbols; nothing yet says what popping does. In Algae, operators are
**inert** — the checker never evaluates them, so ``pop(A, push(A, x, empty(A)))`` does not
quietly collapse to ``empty`` on its own.

Axioms give operators meaning
=============================

Operators earn their meaning from **axioms** — sequents asserted true with no
proof. Two equations are enough to make these symbols behave like a stack:

.. code-block:: alg

   axiom top_push(A : Sort, x : A, s : Stack(A))
     ⊢ top(A, push(A, x, s)) = x;   # the top of a push is what you pushed
   axiom pop_push(A : Sort, x : A, s : Stack(A))
     ⊢ pop(A, push(A, x, s)) = s;   # popping a push undoes it

An axiom is exactly a **zero-premise rule** (recall :doc:`backward-reasoning`):
nothing to establish, so wherever its conclusion matches your goal, it closes it.
Describing a structure by its operations plus a handful of such equations is what
"algebraic specification" means — and because the axioms are all you assume, a
proof from them holds for *every* structure that satisfies them.

How you actually *use* these equations to prove things — closing ``x = x`` and
rewriting one subterm of a goal with an equation — is the subject of the next
chapter.
