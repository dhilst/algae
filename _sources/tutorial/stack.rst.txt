==================
Specifying a stack
==================

Algae isn't just a proof language — it's an *algebraic specification* language.
That means you can describe a data structure by **what its operations do**, not
how they're implemented, and then prove properties that hold for *every*
implementation. Let's specify a classic: the stack.

A stack has four operations — an empty stack, and push / pop / top:

.. code-block:: alg

   sort Stack : Sort → Sort;

   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op pop   : Stack(A) → Stack(A);
   op top   : Stack(A) → A;

Those signatures say what *types* the operations have, but nothing yet about how a
stack *behaves* — with only these, ``pop`` could return anything. The behaviour
lives in the axioms. And remarkably, a stack needs just **two**:

.. code-block:: alg

   axiom top_push(A : Sort, x : A, s : Stack(A))
     ⊢ top(push(x, s)) = x;

   axiom pop_push(A : Sort, x : A, s : Stack(A))
     ⊢ pop(push(x, s)) = s;

Read them aloud and you can *hear* the stackness:

- ``top(push(x, s)) = x`` — whatever you push, you get straight back on top.
- ``pop(push(x, s)) = s`` — pushing then popping leaves the stack untouched.

That's the whole of "last in, first out." The most recent ``push`` is the only
thing ``top`` and ``pop`` can see, and popping it uncovers exactly what was there
before. Every stack law we could want is a *consequence* of these two.

.. note::

   Notice what the axioms **don't** say: nothing about ``top(empty)`` or
   ``pop(empty)``. Algae has no partial functions — ``top`` is total, so
   ``top(empty)`` *is* some element, the axioms just never pin down which. A
   fully-defended spec would use a sum type (``top : Stack(A) → Option(A)``); here
   we keep it lean and simply never reason about the empty case.

Cool proofs, for free
=====================

Now the payoff. Everything below follows from those two axioms alone — for *any*
element type ``A`` and *any* stack ``s``. Press **Check ▶**:

.. code-block:: alg

   import core(forward);

   sort Stack : Sort → Sort;

   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op pop   : Stack(A) → Stack(A);
   op top   : Stack(A) → A;

   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(push(x, s)) = s;

   # 1. The top of a push is exactly what you pushed — the stack below is invisible.
   lemma top_of_two(A : Sort, a b : A, s : Stack(A))
     ⊢ top(push(a, push(b, s))) = a;
   proof
     by top_push(A, a, push(b, s));
   qed;

   # 2. Pop once, and the element that was hidden underneath is now on top.
   lemma top_after_pop(A : Sort, a b : A, s : Stack(A))
     ⊢ top(pop(push(a, push(b, s)))) = b;
   proof
     by forward(Stack(A), pop(push(a, push(b, s))), push(b, s),
                  pop_push(A, a, push(b, s)), top(_) = b)
     then ⊢ top(push(b, s)) = b;
     by top_push(A, b, s);
   qed;

   # 3. Two pushes, two pops, right back where we started.
   lemma pop_twice(A : Sort, a b : A, s : Stack(A))
     ⊢ pop(pop(push(a, push(b, s)))) = s;
   proof
     by forward(Stack(A), pop(push(a, push(b, s))), push(b, s),
                  pop_push(A, a, push(b, s)), pop(_) = s)
     then ⊢ pop(push(b, s)) = s;
     by pop_push(A, b, s);
   qed;

Three obligations, all discharged. Reading them:

- **``top_of_two``** is a one-liner. ``top_push`` says ``top(push(a, _)) = a`` for
  *any* stack in the hole — including ``push(b, s)`` — so the whole thing collapses
  in a single step. The ``b`` and ``s`` underneath never matter to ``top``.
- **``top_after_pop``** is the LIFO story in a proof. We rewrite the inner
  ``pop(push(a, push(b, s)))`` to ``push(b, s)`` with ``forward`` (using
  ``pop_push``) — the motive
  ``top(_) = b`` aims the rewrite at the argument of ``top`` (see
  :doc:`rewrite-reflexivity`) — leaving ``top(push(b, s)) = b``, which is ``top_push`` again.
  So after one pop, ``b`` really is on top.
- **``pop_twice``** chains two rewrites: ``pop_push`` peels the outer ``push(a, …)``
  to reach ``pop(push(b, s))``, and a second ``pop_push`` peels that to ``s``.

None of these mention a concrete stack — no arrays, no linked lists, no code. They
are true of *anything* that satisfies ``top_push`` and ``pop_push``. Bundle those
two axioms into a ``theory Stack`` (see :doc:`theories`) and every one of these
lemmas becomes a guarantee about each of its models. That's the whole idea of
algebraic specification: nail the behaviour down with a handful of equations, and
the proofs come along for the ride.
