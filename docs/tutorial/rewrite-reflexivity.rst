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
   op push : forall (A : Sort) st A * Stack(A) → Stack(A);

   lemma same(A : Sort, x : A, s : Stack(A))
     ⊢ push(A, x, s) = push(A, x, s);
   proof
     by refl(Stack(A), push(A, x, s));
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
runs them and never applies your axioms on its own. So ``pop(A, push(A, x, s))`` is
**not** automatically ``s`` — even though your axiom says they're equal — and
``top(A, push(A, x, s))`` is not automatically ``x``. To the kernel those are just
different symbol trees until *you* apply the equation that relates them.

That's why ``refl`` closes ``push(A, x, s) = push(A, x, s)`` (identical trees) but would
**not** close ``top(A, push(A, x, s)) = x`` (two different trees, equal only *by an
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

.. admonition:: The shortcut — the ``rewrite(…)?`` hole
   :class: tip

   You rarely spell out a whole ``forward`` call by hand. Instead, name the
   equation you want to use inside a **suggestion hole** and let the checker write
   the rest:

   .. code-block:: alg

      by rewrite(<eq>)?;

   where ``<eq>`` is any proof of an equation ``lhs = rhs`` — an axiom, a lemma, or
   a hypothesis in scope. Like every ``?`` hole it does not close the goal; it
   reports a suggestion. To build it, the checker:

   #. reads the equation ``lhs = rhs`` that ``<eq>`` proves;
   #. finds ``lhs`` in the goal and builds the motive ``P`` for you, placing the
      ``_`` at **every** occurrence of that subterm;
   #. infers the sort ``T`` of the two sides;
   #. checks that the resulting ``forward`` step actually applies.

   If it does, it offers the fully-written step as a one-click fix (or you paste
   it yourself):

   .. code-block:: alg

      by forward(T, lhs, rhs, <eq>, P)
      then <context> ⊢ <new goal>;

   The ``<new goal>`` is your goal with ``lhs`` rewritten to ``rhs``, and the
   ``<context>`` is restated in full so nothing is silently dropped. Nothing named
   ``rewrite`` ever reaches the kernel — it expands to ``forward`` at check time,
   and the kernel re-checks that ``forward`` step. So the mechanics below are
   exactly what the hole generates; learn them once and the shortcut is just a
   time-saver.

   If ``lhs`` does not occur in the goal — or the argument is not an equation — the
   hole explains why and offers no fix.

Equational reasoning on the stack
=================================

Here's the smallest real rewrite. Push ``b`` then ``a`` onto the empty stack, pop
once, and the top is ``b``. We know ``pop(A, push(A, a, …)) = …`` from ``pop_push``, so we
rewrite that ``pop(A, push(A, a, …))`` **forward** into the stack underneath, then read
off the top:

.. code-block:: alg

   rule forward(T : Sort, a b : T, eq := a = b, P : T → Prop)
     ⊢ P(b)
     ────────────────────────
     ⊢ P(a)
   end;

   sort Stack : Sort → Sort;
   op empty : forall (A : Sort) st → Stack(A);
   op push  : forall (A : Sort) st A * Stack(A) → Stack(A);
   op pop   : forall (A : Sort) st Stack(A) → Stack(A);
   op top   : forall (A : Sort) st Stack(A) → A;
   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(A, push(A, x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(A, push(A, x, s)) = s;

   lemma one_pop(A : Sort, a b : A)
     ⊢ top(A,
         pop(A, push(A, a, push(A, b, empty(A))))    # this is going to be replaced
       ) = b;
   proof
     by forward(Stack(A),
         pop(A, push(A, a, push(A, b, empty(A)))),   # replace this
         push(A, b, empty(A)),                 # with this
         pop_push(A, a, push(A, b, empty(A))), # using this equation
         top(A, _) = b)                     # at this position
     then ⊢ top(A, push(A, b, empty(A))) = b;     # yielding this
     by top_push(A, b, empty(A));
   qed;

Read the placeholder ``top(A, _) = b`` by plugging each side of the equation
``pop(A, push(A, a, …)) = push(A, b, empty(A))`` into the ``_``:

- ``a`` side in the hole → ``top(A, pop(A, push(A, a, …))) = b`` — our current goal.
- ``b`` side in the hole → ``top(A, push(A, b, empty(A))) = b`` — the goal after the
  rewrite, which ``top_push`` closes.

Everything from ``Stack(A)`` down to the ``top(A, _) = b`` placeholder in that step
is mechanical: it is fixed by the equation ``pop_push(A, a, push(A, b, empty(A)))``
and the goal. That is exactly what the ``rewrite(…)?`` hole works out for you — the
whole ``by forward(…) then …;`` above is what

.. code-block:: alg

   by rewrite(pop_push(A, a, push(A, b, empty(A))))?;

suggests when you run the check.

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
   op empty : forall (A : Sort) st → Stack(A);
   op push  : forall (A : Sort) st A * Stack(A) → Stack(A);
   op pop   : forall (A : Sort) st Stack(A) → Stack(A);
   op top   : forall (A : Sort) st Stack(A) → A;
   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(A, push(A, x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(A, push(A, x, s)) = s;

   lemma three_deep(A : Sort, a b c : A)
     ⊢ top(A, pop(A, pop(A, push(A, a, push(A, b, push(A, c, empty(A))))))) = c;
   proof
     by forward(Stack(A),
         pop(A, push(A, a, push(A, b, push(A, c, empty(A))))),
         push(A, b, push(A, c, empty(A))),
         pop_push(A, a, push(A, b, push(A, c, empty(A)))),
         top(A, pop(A, _)) = c)
     then ⊢ top(A, pop(A, push(A, b, push(A, c, empty(A))))) = c;
     by forward(Stack(A), pop(A, push(A, b, push(A, c, empty(A)))), push(A, c, empty(A)),
                pop_push(A, b, push(A, c, empty(A))), top(A, _) = c)
     then ⊢ top(A, push(A, c, empty(A))) = c;
     by top_push(A, c, empty(A));
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
``top(A, _) = b``:

.. code-block:: text

   error: tactic `forward`: rule conclusion does not match the current goal
     the rule concludes:  (λ (x : Stack(A)) st x = b)(pop(A, push(A, a, push(A, b, empty(A)))))
     but the goal is:     top(A, pop(A, push(A, a, push(A, b, empty(A))))) = b

With ``_ = b`` the hole swallows the ``top(…)`` as well, so filling it produces
``pop(A, push(A, a, …)) = b`` — *not* the goal. When you hit "rule conclusion does not
match the current goal" on a rewrite, the placeholder is almost always the culprit:
move the ``_`` to the subterm you actually mean to touch.

The other direction: ``backward``
=================================

``forward`` follows an equation left to right; ``backward`` runs it the other way,
replacing ``b`` with ``a`` — usually to *expand* a term so another rule can fire.
That move is the everyday shape of an induction step, so we'll meet ``backward``
doing real work in the very next chapter, :doc:`induction`.
