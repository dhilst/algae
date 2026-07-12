=====================
Reasoning about data
=====================

Logic tells you how to combine facts; the data modules tell you how to reason about
*values*. Each one hands you two kinds of rule:

- **case-analysis** rules that mirror a type's constructors — to prove something
  about *any* value, prove it for each way the value could have been built;
- **equations** (axioms) describing what the operations *do*, which you drive with
  ``backward`` / ``forward`` (see :doc:`../rewrite-reflexivity`).

We met one already — ``induction`` in |nat.alg| is exactly the case-analysis rule
for the naturals (base case ``0``, step case ``s(n)``). Everything here is the same
idea for other shapes.

Pairs and sums
==============

|adt.alg| defines ``Pair`` and ``Sum``. A pair is built exactly one way — with
``pair`` — so ``pair_cases`` has a single case (and a single premise, hence
``then``):

.. code-block:: alg

   import adt(pair_cases, refl);

   lemma a_pair_is_itself(A B : Sort, p : Pair(A, B))
     ⊢ p = p;
   proof
     by pair_cases(A, B, p, _ = _)
     then x : A, y : B ⊢ pair(A, B, x, y) = pair(A, B, x, y);
     by refl(Pair(A, B), pair(A, B, x, y));
   qed;

``pair_cases`` replaces the opaque ``p`` with a concrete ``pair(A, B, x, y)`` for fresh
``x``, ``y``. A sum, by contrast, is built *two* ways — ``inl`` or ``inr`` — so
``sum_cases`` gives you two branches:

.. code-block:: alg

   import adt(sum_cases, refl);

   lemma a_sum_is_itself(A B : Sort, s : Sum(A, B))
     ⊢ s = s;
   proof
     by sum_cases(A, B, s, _ = _) cases
       case
         x : A;
         ⊢ inl(A, B, x) = inl(A, B, x);
         by refl(Sum(A, B), inl(A, B, x));
       qed;
       case
         y : B;
         ⊢ inr(A, B, y) = inr(A, B, y);
         by refl(Sum(A, B), inr(A, B, y));
       qed;
     qed;
   qed;

That last argument, ``_ = _``, is the motive again — ``λ k. k = k`` — the property
being proved of the whole value.

Options, results, lists
=======================

The data types follow suit. |option.alg| gives ``option_cases`` (``none`` or
``some(A, x)``):

.. code-block:: alg

   import option(option_cases, refl);

   lemma an_option_is_itself(A : Sort, m : Option(A))
     ⊢ m = m;
   proof
     by option_cases(A, m, _ = _) cases
       case
         ⊢ none(A) = none(A);
         by refl(Option(A), none(A));
       qed;
       case
         x : A;
         ⊢ some(A, x) = some(A, x);
         by refl(Option(A), some(A, x));
       qed;
     qed;
   qed;

|result.alg| mirrors it with ``result_cases`` (``ok(A, E, x)`` or ``err(A, E, e)``), and
|list.alg| gives ``list_induction`` — a *recursive* case analysis, like ``nat``:
the ``cons`` case even hands you an induction hypothesis ``ih`` about the tail.

.. code-block:: alg

   import list(list_induction, refl);

   lemma a_list_is_itself(A : Sort, xs : List(A))
     ⊢ xs = xs;
   proof
     by list_induction(A, xs, _ = _) cases
       case
         ⊢ nil(A) = nil(A);
         by refl(List(A), nil(A));
       qed;
       case
         x : A;
         rest : List(A);
         ih := rest = rest;
         ⊢ cons(A, x, rest) = cons(A, x, rest);
         by refl(List(A), cons(A, x, rest));
       qed;
     qed;
   qed;

The equations
=============

Case rules take values apart; the **equation** axioms say what the operations
compute to. |option.alg|'s ``bind_some`` is a fact you can apply directly — it says
binding into a ``some`` just runs the function:

.. code-block:: alg

   import option;

   lemma bind_runs_the_function(A B : Sort, x : A, f : A → Option(B))
     ⊢ bind(A, B, some(A, x), f) = f(x);
   proof
     by bind_some(A, B, x, f);
   qed;

That's the same move the monad-law proofs in :doc:`../theories` were built from:
case-split with ``option_cases``, then rewrite with ``bind_none`` / ``bind_some``
until both sides meet. Every data module is this pair — constructors to split on,
equations to rewrite with.

.. admonition:: Your turn
   :class: tip

   Binding into ``none`` throws the function away. Prove it — the axiom you need is
   ``bind_none``.

   .. code-block:: alg

      import option;

      lemma bind_of_none(A B : Sort, g : A → Option(B))
        ⊢ bind(A, B, none(A), g) = none(B);
      proof
        by wip(?goal);
      wip;

   .. hint::

      ``bind_none(A, B, g)`` proves ``bind(A, B, none(A), g) = none(B)`` outright — it's a
      premise-free fact, so a single ``by bind_none(A, B, g);`` closes the goal, no
      ``then`` needed.

That's the tour. You've now met the whole vocabulary: equality and rewriting, the
logical connectives, the quantifiers, and case analysis over every data type in the
library. Everything else in Algae — the ``option``/``list``/``result`` monad
proofs, the ``group`` hierarchy, whatever you build next — is these same rules,
chained a little longer. Open the modules, read a proof, and try to reprove it
yourself. The kernel is patient, and now, so are you.
