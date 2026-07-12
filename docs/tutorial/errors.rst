=========================
Specifying error behavior
=========================

Remember the warning back in :doc:`specs`, that our stack specification was
**incomplete**? Here is exactly how. What is ``top(A, empty(A))`` — the top of an
*empty* stack? The two axioms never say. A programmer implementing that spec would
have to decide something on the spot — perhaps throw an exception — for a case the
specification simply left open. We usually want to *avoid* runtime errors, or at
the very least pin down in the specification where they may occur.

So this is a real gap. Algae has no partial functions — ``top`` is total, so
``top(A, empty(A))`` *is* some element, we just never said which. A caller who pops one
element too many deserves a defined answer, not a shrug.

Algae's motto is *failure is modeled with sum types*. The standard library gives
this shape a name: ``Result(A, E)`` — the type of "an ``A`` on success, *or* an
``E`` on failure," built from two constructors,

.. code-block:: alg

   ok  : forall (A E : Sort) st A → Result(A, E);   # success, carrying an A
   err : forall (A E : Sort) st E → Result(A, E);   # failure, carrying an E

So we give ``pop`` and ``top`` a second possible outcome — an ``Error`` — by
returning a ``Result`` and specifying the empty case outright:

.. code-block:: alg

   import result(ok, err, result_cases);

   sort Error : Sort;

   op underflow : → Error;   # the one way our stack operations can fail
   op pop : forall (A : Sort) st Stack(A) → Result(Stack(A), Error);
   op top : forall (A : Sort) st Stack(A) → Result(A, Error);

   axiom pop_empty(A : Sort)
     ⊢ pop(A, empty(A)) = err(Stack(A), Error, underflow);
   axiom top_empty(A : Sort)
     ⊢ top(A, empty(A)) = err(A, Error, underflow);

The return types are now **sum types**: ``pop`` yields "a stack *or* an error,"
written ``Result(Stack(A), Error)``. Unlike an untyped model, a plain value does
**not** silently slip into the sum — you say *which* outcome it is with ``ok`` or
``err``:

- ``err(Stack(A), Error, underflow)`` tags the ``Error`` value ``underflow`` as the
  *failure* branch, giving it type ``Result(Stack(A), Error)``.
- On success you use ``ok``: a real result ``s : Stack(A)`` becomes
  ``ok(Stack(A), Error, s) : Result(Stack(A), Error)``. The old success laws
  survive, reworded to say "on success, you get ``ok`` of your value back."

(The ``import result(ok, err, result_cases);`` is what brings the ``Result`` type,
its ``ok`` / ``err`` constructors, and the ``result_cases`` eliminator into scope.)

The explicit ``ok`` / ``err`` are the price of honesty: the type now records
*which* outcome happened, and every use site has to say so.

Here's the whole thing, error-aware, still verifying:

.. code-block:: alg

   import result(ok, err, result_cases);

   sort Stack : Sort → Sort;
   sort Error : Sort;

   op empty     : forall (A : Sort) st → Stack(A);
   op push      : forall (A : Sort) st A * Stack(A) → Stack(A);
   op underflow : → Error;
   op pop       : forall (A : Sort) st Stack(A) → Result(Stack(A), Error);
   op top       : forall (A : Sort) st Stack(A) → Result(A, Error);

   axiom top_push(A : Sort, x : A, s : Stack(A))
     ⊢ top(A, push(A, x, s)) = ok(A, Error, x);
   axiom pop_push(A : Sort, x : A, s : Stack(A))
     ⊢ pop(A, push(A, x, s)) = ok(Stack(A), Error, s);
   axiom pop_empty(A : Sort)
     ⊢ pop(A, empty(A)) = err(Stack(A), Error, underflow);
   axiom top_empty(A : Sort)
     ⊢ top(A, empty(A)) = err(A, Error, underflow);

   # empty is now fully specified: peeking it is an error, not a mystery.
   lemma peek_empty(A : Sort)
     ⊢ top(A, empty(A)) = err(A, Error, underflow);
   proof
     by top_empty(A);
   qed;

   # the success law: peeking a push returns ok of the pushed value.
   lemma top_of_push(A : Sort, x : A, s : Stack(A))
     ⊢ top(A, push(A, x, s)) = ok(A, Error, x);
   proof
     by top_push(A, x, s);
   qed;

   # pop after push returns the same stack — a *successful* ok of s.
   lemma pop_of_push(A : Sort, a : A, s : Stack(A))
     ⊢ pop(A, push(A, a, s)) = ok(Stack(A), Error, s);
   proof
     by pop_push(A, a, s);
   qed;

- **``peek_empty``** is the point of the whole exercise: ``top(A, empty(A))`` now has an
  answer, ``err(…, underflow)``, provable in one step. The gap is closed.
- **``top_of_push``** confirms nothing regressed — the success law is exactly as
  before, now stated as an explicit ``ok`` result.
- **``pop_of_push``** is the round-trip law: for *any* stack ``s``, pushing ``a``
  then popping returns ``ok(Stack(A), Error, s)`` — a *tagged success* carrying the
  same stack ``s`` you started with.

The tradeoff
============

Specifying errors this way buys you **totality and honesty**: there are no
undefined corners left, ``pop(A, empty(A))`` genuinely equals ``err(…, underflow)``,
and any caller can tell success from failure by looking at which constructor they
got.

But you pay for it, and the types now make you pay up front. Once ``pop`` returns
``Result(Stack(A), Error)``, you can no longer write ``top(A, pop(A, s))`` — ``top``
wants a ``Stack(A)``, not a ``Result(Stack(A), Error)``, so the composition simply
does not type-check. To chain another operation you must **case-split** the result
first (with ``result_cases``): if it is ``ok`` of a stack, carry on; if it is
``err`` of an error, propagate. The tidy equations sprout side conditions, and
``pop(A, pop(A, s))`` stops being something you can write without first asking "did
the first pop fail?"

That's the perennial bargain. Leave failure unspecified and the algebra stays
clean but the spec lies by omission. Track it in the types and the spec tells the
whole truth, but the plumbing multiplies. For a stack — where popping too far is
an everyday mistake — paying for honesty is usually worth it. For an operation you
can *prove* never fails, the sum type is just noise you'll spend proofs peeling
back off. Choosing where to draw that line is a real part of writing a
specification.
