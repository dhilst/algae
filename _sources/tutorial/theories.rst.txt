==========================
Theories, laws, and models
==========================

Single facts are nice, but real structure comes from **theories** and the
**models** that satisfy them. A theory is a parameterized interface plus a list
of **laws** (propositions its implementers must prove). ``group.alg`` builds the
classic algebra hierarchy, each theory ``include``-ing the previous one and piling
on laws:

.. code-block:: alg

   theory Monoid(
     S : Sort,
     mul : S * S → S,
     e : S
   ) laws
     include Semigroup(S, mul);            # associativity, inherited

     law left_identity(x : S)   ⊢ mul(e, x) = x;
     law right_identity(x : S)  ⊢ mul(x, e) = x;
   qed;

A **model** claims specific operators satisfy a theory, and must prove every law
as an obligation. Here's the ``Monad`` interface from ``monad.alg``:

.. code-block:: alg

   theory Monad(
     A B C : Sort,
     M : Sort → Sort,
     return : A → M(A),
     bind : M(A) * (A → M(B)) → M(B)
   ) laws
     law left_identity(x : A, f : A → M(B))  ⊢ bind(return(x), f) = f(x);
     law right_identity(m : M(A))            ⊢ bind(m, return) = m;
     law associativity(m : M(A), f : A → M(B), g : B → M(C))
       ⊢ bind(bind(m, f), g) = bind(m, λ (x : A) st bind(f(x), g));
   qed;

``option.alg``, ``list.alg``, and ``result.alg`` each ship a verified ``model``
proving their type satisfies ``Monad``. Let's build a smaller one, end to end, that
you can actually run.

Remember the stack from :doc:`stack`? Those two axioms are really an *interface* —
any type with ``push`` / ``pop`` / ``top`` obeying them is a stack. So make that a
theory, then prove our concrete stack is a **model** of it:

.. code-block:: alg

   import core;

   sort Stack : Sort → Sort;
   op empty : → Stack(A);
   op push  : A * Stack(A) → Stack(A);
   op pop   : Stack(A) → Stack(A);
   op top   : Stack(A) → A;

   axiom top_ax(A : Sort, x : A, s : Stack(A))  ⊢ top(push(x, s)) = x;
   axiom pop_ax(A : Sort, x : A, s : Stack(A))  ⊢ pop(push(x, s)) = s;

   # the interface: any S with these operations obeying these laws is a stack
   theory StackSpec(
     A : Sort,
     S : Sort → Sort,
     e : S(A),
     psh : A * S(A) → S(A),
     pp : S(A) → S(A),
     tp : S(A) → A
   ) laws
     law top_law(x : A, s : S(A))  ⊢ tp(psh(x, s)) = x;
     law pop_law(x : A, s : S(A))  ⊢ pp(psh(x, s)) = s;
   qed;

   # the claim: our concrete operations are a stack
   model ConcreteStack satisfies StackSpec(A, Stack, empty, push, pop, top) iff props
     law top_law;
     proof
       by top_ax(A, x, s);
     qed;

     law pop_law;
     proof
       by pop_ax(A, x, s);
     qed;
   qed;

Read the ``model`` header as *binding* each theory parameter to something concrete:
the constructor ``S`` becomes ``Stack``, ``psh`` becomes ``push``, and so on. Then
``iff props`` opens the obligations — one ``law <name>; proof … qed;`` per law in
the theory — and each is proved just like a lemma. Here every proof is a one-liner,
because ``StackSpec``'s laws are exactly our two axioms. Press **Check ▶**: two
obligations discharged, and ``ConcreteStack`` is certified a stack.

Every model has this shape, however big. ``option.alg``'s ``OptionMonad`` is the
same skeleton with three richer proofs — each threading ``rewrite_r`` to reach its
equality, the ``defeq`` discipline from :doc:`first-proofs` at scale.

Imports and the standard library
================================

``import module;`` brings in **everything** a module declares — its sorts,
operators, axioms, and rules. ``import module(name, …)`` selects specific names,
and ``import module(name as alias)`` renames. Either way the module's operators
come along (which is why ``import nat;`` let us write ``0`` and ``+``).

The standard library lives in ``algae/stdlib/v1/``:

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - module
     - what it provides
   * - ``core``
     - equality (``refl``, ``symmetry``, ``rewrite_r`` / ``rewrite_l``), logic, quantifiers
   * - ``nat``
     - ``Nat``, ``+``, ``*``, and ``induction``
   * - ``option``, ``result``, ``list``
     - data types with their ``Monad`` models
   * - ``monad``
     - the ``Functor`` / ``Applicative`` / ``Monad`` theories
   * - ``group``
     - the ``Magma`` → … → ``AbelianGroup`` hierarchy
   * - ``adt``
     - algebraic-datatype scaffolding

Verify the whole library in one go:

.. code-block:: sh

   cargo run -p algae-cli -- verify algae/stdlib/v1/

Where to go next
================

- ``algae/stdlib/v1/`` — worked, verified modules to read and imitate.
- ``lang-specs/spec.md`` (in the repository) — the precise grammar and static
  semantics, when you want the letter of the law.
- ``tests/accept/`` — one minimal proof per inference rule, if you like your
  examples bite-sized.

Now go break some proofs. The kernel is waiting.
