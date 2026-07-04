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

``option.alg`` proves ``Option`` is a lawful monad with a ``model`` block. Its
shape is: name the theory, bind each parameter to a concrete operator, then prove
every law just like a lemma (this listing elides the proof bodies):

.. code-block:: text

   model OptionMonad satisfies Monad(A, B, C, Option, return, bind) iff props
     law left_identity;   proof … qed;
     law right_identity;  proof … qed;
     law associativity;   proof … qed;
   qed;

Here's that first law — ``bind(return(x), f) = f(x)`` — as a standalone lemma you
can actually run (inside the model it's the body of ``law left_identity;``). Since
``return(x)`` equals ``some(x)`` only *through* the axiom ``return_def`` — never by
silent computation — the proof **rewrites** ``return(x)`` to ``some(x)`` with
``rewrite_r``, then finishes with ``bind_some``:

.. code-block:: alg

   import option;
   import core(rewrite_r);

   lemma option_left_identity(A B : Sort, x : A, f : A → Option(B))
     ⊢ bind(return(x), f) = f(x);
   proof
     by rewrite_r(
       Option(A),
       return(x), some(x),
       return_def(A, x),                       # return(x) = some(x)
       λ (o : Option(A)) st bind(o, f) = f(x)
     )
     then ⊢ bind(some(x), f) = f(x);
     by bind_some(A, B, x, f);
   qed;

A model bundles three proofs like this — one per law — and, once verified,
certifies ``Option`` as a monad. It's the ``defeq`` discipline from
:doc:`first-proofs` at scale: every monad-law proof in ``option.alg``,
``list.alg``, and ``result.alg`` reaches its equalities through explicit
``rewrite_r`` / ``rewrite_l`` steps — never by silent evaluation.

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
