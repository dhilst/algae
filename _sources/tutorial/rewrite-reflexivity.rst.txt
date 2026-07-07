===============================
Reflexivity and rewriting
===============================

Equations are the heart of an algebraic specification, so this chapter is about
*using* them: **reflexivity** to close ``x = x``, and the two **rewrite** rules to
apply an equation to one part of a goal. We'll reason about the stack from
:doc:`specs`.

Reflexivity
===========

``refl`` proves anything equal to itself. It's an axiom — zero premises — so it
closes a goal outright. We'll write it out in the buffer rather than importing it:

.. code-block:: alg

   axiom refl(T : Sort, x : T)
     ⊢ x = x;

   sort Stack : Sort → Sort;
   op push : A * Stack(A) → Stack(A);

   lemma same(A : Sort, x : A, s : Stack(A))
     ⊢ push(x, s) = push(x, s);
   proof
     by refl(Stack(A), push(x, s));
   qed;

That looks trivial, and it is — but note *when* it works, because that's the one
thing newcomers trip on.

What "the same term" means
==========================

Algae has a built-in notion of when two terms are **the same** — it's called
**definitional equality**, and it is deliberately tiny. Two terms are definitionally
equal only when they become *literally identical* after two harmless clean-ups:

- **Renaming a local variable.** The bound name in a ``∀``, ``∃``, or ``λ`` is
  arbitrary — ``∀ (x : T) st x = x`` and ``∀ (y : T) st y = y`` are the same
  statement. (Logicians call this *α*.)
- **Carrying out a function application.** If you apply a ``λ`` (a little inline
  function) to an argument, you may substitute the argument in. So
  ``(λ (x : T) st x = x)(a)`` *is* ``a = a``. (This one is called *β*.)

And **that's all** — "α/β equivalence only." Crucially, the *operators* you
declared (``push``, ``pop``, ``top``, ``+`` …) are **inert**: the kernel never
runs them and never applies your axioms on its own. So ``pop(push(x, s))`` is
**not** automatically ``s`` — even though your axiom says they're equal — and
``top(push(x, s))`` is not automatically ``x``. To the kernel those are just
different symbol trees until *you* apply the equation that relates them.

That's why ``refl`` closes ``push(x, s) = push(x, s)`` (identical trees) but would
**not** close ``top(push(x, s)) = x`` (two different trees, equal only *by an
axiom*). Bridging that gap is exactly what rewriting is for.

The rewrite rules and the placeholder
======================================

The workhorses of equational reasoning are the two congruence rules. Each takes an
equation and rewrites **one chosen subterm** of the goal:

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

The interesting argument is the last one, ``P`` — a function ``T → Prop`` that is
your goal *with a hole in it*. That hole marks exactly which subterm the equation
lands on; it is a **placeholder** for the term being replaced. The names say which
way the equation runs, reading it as ``a = b``:

- **``forward``** replaces ``a`` with ``b`` in the goal — following the equation
  left to right, the way you read it.
- **``backward``** replaces ``b`` with ``a`` — using the equation right to left.

You write the placeholder with ``_``. Writing the whole function as
``λ (x : T) st …`` gets old fast, so ``_`` is sugar for it: ``top(_) = b`` means
``λ (x : Stack(A)) st top(x) = b``. The ``_`` is the slot the equation's two sides
plug into.

.. admonition:: Sugar on the way
   :class: note

   A lighter surface syntax for rewriting is planned for Algae — one that will
   read closer to "rewrite this equation here" and expand to an application of the
   ``forward`` rule under the hood. The *mechanics* won't change: it will still be
   ``forward`` doing the work, and everything you learn here will carry straight
   over.

Equational reasoning on the stack
=================================

Here's the smallest real rewrite. Push ``b`` then ``a`` onto the empty stack, pop
once, and the top is ``b``. We know ``pop(push(a, …)) = …`` from ``pop_push``, so we
rewrite that ``pop(push(a, …))`` **forward** into the stack underneath, then read
off the top:

.. code-block:: alg

   rule forward(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(b)
     ────────────────────────
     ⊢ P(a)
   end;

   sort Stack : Sort → Sort;
   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op pop   : Stack(A) → Stack(A);
   op top   : Stack(A) → A;
   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(push(x, s)) = s;

   lemma one_pop(A : Sort, a b : A)
     ⊢ top(
         pop(push(a, push(b, empty)))    # this is going to be replaced
       ) = b;
   proof
     by forward(Stack(A),
         pop(push(a, push(b, empty))),   # replace this
         push(b, empty),                 # with this
         pop_push(A, a, push(b, empty)), # using this equation
         top(_) = b)                     # at this position
     then ⊢ top(push(b, empty)) = b;     # yielding this
     by top_push(A, b, empty);
   qed;

Read the placeholder ``top(_) = b`` by plugging each side of the equation
``pop(push(a, …)) = push(b, empty)`` into the ``_``:

- ``a`` side in the hole → ``top(pop(push(a, …))) = b`` — our current goal.
- ``b`` side in the hole → ``top(push(b, empty)) = b`` — the goal after the
  rewrite, which ``top_push`` closes.

Deeper stacks work the same way — just more rewrites. Three pushes and two pops:

*Don't worry — you can't read this. This is exactly why we need induction, which
we'll cover later.*

.. code-block:: alg

   rule forward(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(b)
     ────────────────────────
     ⊢ P(a)
   end;

   sort Stack : Sort → Sort;
   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op pop   : Stack(A) → Stack(A);
   op top   : Stack(A) → A;
   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(push(x, s)) = s;

   lemma three_deep(A : Sort, a b c : A)
     ⊢ top(pop(pop(push(a, push(b, push(c, empty)))))) = c;
   proof
     by forward(Stack(A),
         pop(push(a, push(b, push(c, empty)))),
         push(b, push(c, empty)),
         pop_push(A, a, push(b, push(c, empty))),
         top(pop(_)) = c)
     then ⊢ top(pop(push(b, push(c, empty)))) = c;
     by forward(Stack(A), pop(push(b, push(c, empty))), push(c, empty),
                pop_push(A, b, push(c, empty)), top(_) = c)
     then ⊢ top(push(c, empty)) = c;
     by top_push(A, c, empty);
   qed;

.. admonition:: This only reaches *fixed* depths
   :class: warning

   Notice what we can and can't do. We can prove the pop-through-``n``-pushes fact
   for ``n = 1``, ``n = 2``, ``n = 3`` — but each is a *separate* proof, spelled
   out one rewrite per push. There is no way, with rewriting alone, to prove the
   statement for a stack of **arbitrary** length in one go — you'd need to write
   infinitely many rewrites. Reasoning about arbitrarily large values is exactly
   what **induction** is for, and it's the subject of the next chapter.

When the placeholder misses
===========================

The placeholder has to reproduce the goal when the equation's ``a`` side fills the
hole. Aim it at the wrong subterm and you get a very common error. In ``one_pop``,
suppose we wrote ``_ = b`` — a hole over the *whole* left side — instead of
``top(_) = b``:

.. code-block:: text

   error: tactic `forward`: rule conclusion does not match the current goal
     the rule concludes:  (λ (x : Stack(A)) st x = b)(pop(push(a, push(b, empty))))
     but the goal is:     top(pop(push(a, push(b, empty)))) = b

With ``_ = b`` the hole swallows the ``top(…)`` as well, so filling it produces
``pop(push(a, …)) = b`` — *not* the goal. When you hit "rule conclusion does not
match the current goal" on a rewrite, the placeholder is almost always the culprit:
move the ``_`` to the subterm you actually mean to touch.

The other direction: ``backward``
=================================

``forward`` follows an equation left to right; ``backward`` runs it the other way,
replacing ``b`` with ``a`` — usually to *expand* a term so another rule can fire.
That move is the everyday shape of an induction step, so we'll meet ``backward``
doing real work in the very next chapter, :doc:`induction`.
