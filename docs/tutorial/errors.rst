=========================
Specifying error behavior
=========================

Remember the warning back in :doc:`specs`, that our stack specification was
**incomplete**? Here is exactly how. What is ``top(empty)`` — the top of an
*empty* stack? The two axioms never say. A programmer implementing that spec would
have to decide something on the spot — perhaps throw an exception — for a case the
specification simply left open. We usually want to *avoid* runtime errors, or at
the very least pin down in the specification where they may occur.

So this is a real gap. Algae has no partial functions — ``top`` is total, so
``top(empty)`` *is* some element, we just never said which. A caller who pops one
element too many deserves a defined answer, not a shrug.

Algae's motto is *failure is modeled with sum types*. So we give ``pop`` and
``top`` a second possible outcome — an ``Error`` — and specify the empty case
outright:

.. code-block:: alg

   sort Error : Sort;

   op err : → Error;
   op pop : Stack(A) → Stack(A) | Error;
   op top : Stack(A) → A | Error;

   axiom pop_empty(A : Sort)  ⊢ pop(empty) = err;
   axiom top_empty(A : Sort)  ⊢ top(empty) = err;

The return types are now **sum types**: ``pop`` yields "a stack *or* an error,"
written ``Stack(A) | Error``. Two things make this fit together:

- ``pop(empty) = err`` type-checks because a value of a *summand* injects into the
  sum: ``err : Error`` sits happily where a ``Stack(A) | Error`` is wanted.
- The old success laws survive untouched, for the same reason. ``pop(push(x, s)) =
  s`` still holds — ``s : Stack(A)`` injects into ``Stack(A) | Error`` — it just
  now reads as "on success, you get ``s`` back." Nothing we proved before breaks.

Here's the whole thing, error-aware, still verifying:

.. code-block:: alg

   import core(forward);

   sort Stack : Sort → Sort;
   sort Error : Sort;

   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op err   : → Error;
   op pop   : Stack(A) → Stack(A) | Error;
   op top   : Stack(A) → A | Error;

   axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(push(x, s)) = s;
   axiom pop_empty(A : Sort)  ⊢ pop(empty) = err;
   axiom top_empty(A : Sort)  ⊢ top(empty) = err;

   # empty is now fully specified: peeking it is an error, not a mystery.
   lemma peek_empty(A : Sort)  ⊢ top(empty) = err;
   proof
     by top_empty(A);
   qed;

   # the success laws still hold — an A injects into A | Error.
   lemma top_of_push(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   proof
     by top_push(A, x, s);
   qed;

   # push one, pop it, then peek: the error surfaces through the composition.
   lemma push_pop_peek(A : Sort, a : A)  ⊢ top(pop(push(a, empty))) = err;
   proof
     by forward(Stack(A), pop(push(a, empty)), empty,
                  pop_push(A, a, empty), top(_) = err)
     then ⊢ top(empty) = err;
     by top_empty(A);
   qed;

- **``peek_empty``** is the point of the whole exercise: ``top(empty)`` now has an
  answer, ``err``, provable in one step. The gap is closed.
- **``top_of_push``** confirms nothing regressed — the success law is exactly as
  before.
- **``push_pop_peek``** chains operations: push ``a``, pop it back to ``empty``,
  then peek — and the error propagates out. We rewrite ``pop(push(a, empty))`` to
  ``empty`` and finish with ``top_empty``.

The tradeoff
============

Specifying errors this way buys you **totality and honesty**: there are no
undefined corners left, ``pop(empty)`` genuinely equals ``err``, and any caller
can tell success from failure by looking at which side of the sum they got.

But you pay for it. Once ``pop`` returns ``Stack(A) | Error``, that error rides
along through every composition. ``push_pop_peek`` only went through because the
inner ``pop`` demonstrably *succeeded* (it produced ``empty``, a real stack) — so
the injection carried us past it. Feed a genuinely-might-be-error result into the
next operation and a rigorous spec has to **case-split**: if it errored, propagate;
otherwise carry on. The tidy equations start sprouting side conditions, and
``pop(pop(s))`` stops being something you can write without first asking "did the
first pop fail?"

That's the perennial bargain. Leave failure unspecified and the algebra stays
clean but the spec lies by omission. Track it in the types and the spec tells the
whole truth, but the plumbing multiplies. For a stack — where popping too far is
an everyday mistake — paying for honesty is usually worth it. For an operation you
can *prove* never fails, the sum type is just noise you'll spend proofs peeling
back off. Choosing where to draw that line is a real part of writing a
specification.
