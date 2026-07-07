===============================
Proof techniques crash course
===============================

Every proof you'll ever write is some mix of a small number of standard *moves*.
This chapter is a fast reference: each section names a classic technique, says how
it maps onto an Algae rule, and then hands you **three exercises** to drill it.

Each exercise is a live editor seeded with a hole. Press **Check ▶** to see the
open goal, **click** the hole (or press **Ctrl-Space**) for suggestions, and read
the hint if you're stuck. Everything here uses only ``core`` — the rules you met
in :doc:`inference-rules`, :doc:`backward-reasoning`, and the tour
(:doc:`tour/logic`, :doc:`tour/quantifiers`).

Here's one worked all the way through, so you know what a finished answer looks
like — the *and*-introduction from :doc:`backward-reasoning`:

.. code-block:: alg

   import core(and_intro);

   lemma both(A B : Prop, x := A, y := B)
     ⊢ A ∧ B;
   proof
     by and_intro(A, B) cases
       case ⊢ A; by x; qed;
       case ⊢ B; by y; qed;
     qed;
   qed;

Now the techniques.

1. Direct proof
===============

**Goal:** prove ``P``. **Method:** apply a rule whose conclusion matches ``P``,
then discharge whatever it leaves from your assumptions. A premise-free step (an
axiom or an assumption ``by h``) closes a goal outright.

.. admonition:: Exercises
   :class: tip

   **1a.** From a proof of ``A``, build ``A ∨ B`` (the *left* injection).

   .. code-block:: alg

      import core(or_intro_left);

      lemma inject_left(A B : Prop, x := A)
        ⊢ A ∨ B;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``or_intro_left(A, B)`` proves ``A ∨ B`` from ``⊢ A`` — one premise,
      so ``then ⊢ A;``, closed ``by x``.

   **1b.** Same, but the *right* injection: from a proof of ``B``, build ``A ∨ B``.

   .. code-block:: alg

      import core(or_intro_right);

      lemma inject_right(A B : Prop, y := B)
        ⊢ A ∨ B;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``or_intro_right(A, B)`` proves ``A ∨ B`` from ``⊢ B``. ``then ⊢ B;``
      then ``by y``.

   **1c.** Take a conjunction apart: from a proof of ``A ∧ B``, get ``A``.

   .. code-block:: alg

      import core(and_left);

      lemma take_left(A B : Prop, h := A ∧ B)
        ⊢ A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``and_left(A, B)`` concludes ``⊢ A`` from its one premise ``⊢ A ∧ B``.
      ``then ⊢ A ∧ B;`` and close ``by h``.

2. Conditional proof (implication)
==================================

**Goal:** prove ``P ⇒ Q``. **Method:** assume ``P``, then prove ``Q``.
``implication_intro`` does exactly this — it introduces the antecedent as a
named hypothesis after its premise ``P := P ⊢ Q``.

.. admonition:: Exercises
   :class: tip

   **2a.** Prove the identity implication ``A ⇒ A``.

   .. code-block:: alg

      import core(implication_intro);

      lemma id(A : Prop)
        ⊢ A ⇒ A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``by implication_intro(A, A) then P := A ⊢ A;`` leaves you assuming
      ``P : A`` and proving ``A`` — close it ``by P``.

   **2b.** If you already hold a proof of ``B``, then ``A ⇒ B`` for any ``A``.

   .. code-block:: alg

      import core(implication_intro);

      lemma const_imp(A B : Prop, q := B)
        ⊢ A ⇒ B;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``implication_intro(A, B) then P := A ⊢ B;`` — ignore ``P`` and close
      ``by q``.

   **2c.** Prove ``(A ∧ B) ⇒ A`` — assume the conjunction, then project.

   .. code-block:: alg

      import core(implication_intro, and_left);

      lemma proj(A B : Prop)
        ⊢ (A ∧ B) ⇒ A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``implication_intro(A ∧ B, A) then P := A ∧ B ⊢ A;`` then
      ``by and_left(A, B) then ⊢ A ∧ B; by P;``.

3. Proof by cases
=================

**Goal:** prove ``P``. **Method:** split into all possible cases and prove ``P``
in each. When the "all possible cases" is a disjunction ``A ∨ B`` you have,
``or_elim`` is the tool: it gives you two branches, one assuming ``A`` and one
assuming ``B``, both aiming at the same goal.

.. admonition:: Exercises
   :class: tip

   **3a.** Disjunction commutes: from ``A ∨ B``, prove ``B ∨ A``.

   .. code-block:: alg

      import core(or_elim, or_intro_left, or_intro_right);

      lemma or_comm(A B : Prop, d := A ∨ B)
        ⊢ B ∨ A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``by or_elim(A, B, B ∨ A) cases`` gives three goals: re-prove
      ``A ∨ B`` (``by d``), then a branch ``P := A`` (build ``B ∨ A`` with
      ``or_intro_right``) and a branch ``Q := B`` (with ``or_intro_left``).

   **3b.** A trivial-looking but instructive one: from ``A ∨ A``, prove ``A``.

   .. code-block:: alg

      import core(or_elim);

      lemma idem(A : Prop, d := A ∨ A)
        ⊢ A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``or_elim(A, A, A) cases`` — feed ``d`` for the disjunction, and each
      branch hands you an assumption of ``A`` to close with.

   **3c.** The constructive dilemma: from ``A ∨ B``, ``A ⇒ C`` and ``B ⇒ C``,
   conclude ``C``.

   .. code-block:: alg

      import core(or_elim, implication_elim);

      lemma dilemma(A B C : Prop, d := A ∨ B, f := A ⇒ C, g := B ⇒ C)
        ⊢ C;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``or_elim(A, B, C) cases``. In the ``P := A`` branch run
      ``implication_elim(A, C)`` against ``f`` and ``P``; symmetrically use ``g``
      and ``Q`` in the ``Q := B`` branch.

4. Mathematical induction
=========================

**Goal:** prove ``∀ n : Nat. P(n)``. **Method:** prove ``P(0)`` (the *base
case*), then assume ``P(k)`` and prove ``P(s(k))`` (the *step case*). The
``induction`` rule from ``nat`` has exactly those two premises, so it splits into
two ``case`` s. Here's the classic — ``n + 0 = n`` — worked in full; the
exercises ask you to rebuild each part.

.. code-block:: alg

   import nat;
   import core(backward);

   lemma add_zero_right
     ⊢ ∀ (n : Nat) st n + 0 = n;
   proof
     by induction(_ + 0 = _) cases
       case ⊢ 0 + 0 = 0;
         by add_zero_left(0);
       qed;
       case k : Nat; ih := k + 0 = k; ⊢ s(k) + 0 = s(k);
         by backward(Nat, k + 0, k, ih, s(k) + 0 = s(_))
         then ⊢ s(k) + 0 = s(k + 0);
         by add_succ_left(k, 0);
       qed;
     qed;
   qed;

.. admonition:: Exercises
   :class: tip

   **4a.** Fill in only the **base case** — the step is done for you.

   .. code-block:: alg

      import nat;
      import core(backward);

      lemma add_zero_right
        ⊢ ∀ (n : Nat) st n + 0 = n;
      proof
        by induction(_ + 0 = _) cases
          case ⊢ 0 + 0 = 0;
            by wip(?base);
          wip;
          case k : Nat; ih := k + 0 = k; ⊢ s(k) + 0 = s(k);
            by backward(Nat, k + 0, k, ih, s(k) + 0 = s(_))
            then ⊢ s(k) + 0 = s(k + 0);
            by add_succ_left(k, 0);
          qed;
        qed;
      qed;

   .. hint:: ``0 + 0 = 0`` is exactly ``add_zero_left(0)`` — a premise-free axiom,
      so ``by add_zero_left(0);`` closes it (change the branch's ``wip`` to
      ``qed``).

   **4b.** Now fill in only the **step case** — the base is done.

   .. code-block:: alg

      import nat;
      import core(backward);

      lemma add_zero_right
        ⊢ ∀ (n : Nat) st n + 0 = n;
      proof
        by induction(_ + 0 = _) cases
          case ⊢ 0 + 0 = 0;
            by add_zero_left(0);
          qed;
          case k : Nat; ih := k + 0 = k; ⊢ s(k) + 0 = s(k);
            by wip(?step);
          wip;
        qed;
      qed;

   .. hint:: Rewrite the ``k`` inside ``s(_)`` into ``k + 0`` with
      ``backward(Nat, k + 0, k, ih, s(k) + 0 = s(_))``, ``then ⊢ s(k) + 0 =
      s(k + 0);``, and finish ``by add_succ_left(k, 0);``.

   **4c.** From scratch: prove it with nothing but the induction skeleton.

   .. code-block:: alg

      import nat;
      import core(backward);

      lemma add_zero_right
        ⊢ ∀ (n : Nat) st n + 0 = n;
      proof
        by wip(?goal);
      wip;

   .. hint:: Start ``by induction(_ + 0 = _) cases``, then assemble the two
      branches from 4a and 4b.

5. Proof by contradiction
=========================

**Goal:** prove ``¬P``. **Method:** assume ``P``, and derive absurdity
(``False``). That's ``negation_intro`` — it assumes ``P`` and asks you to reach
``False``. Two more tools travel with it: ``negation_elim`` turns a proof of ``P``
*and* ``¬P`` into ``False``, and ``false_elim`` turns ``False`` into anything at
all (the principle of explosion).

.. admonition:: Algae's logic is intuitionistic
   :class: note

   Classic "proof by contradiction" sometimes means proving a *positive* ``P`` by
   assuming ``¬P`` and deriving ``False``. That step needs the law of excluded
   middle, which ``core`` deliberately does **not** ship — so here contradiction
   proves **negations** (``¬P``), and explosion carries a contradiction to any
   goal. (This is also why the next section has no exercises.)

.. admonition:: Exercises
   :class: tip

   **5a.** From ``A ⇒ False``, prove ``¬A``.

   .. code-block:: alg

      import core(negation_intro, implication_elim);

      lemma neg_from_imp(A : Prop, f := A ⇒ False)
        ⊢ ¬A;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``negation_intro(A) then P := A ⊢ False;`` then run
      ``implication_elim(A, False)`` against ``f`` and ``P``.

   **5b.** Explosion: from a proof of ``A`` and a proof of ``¬A``, prove any ``C``.

   .. code-block:: alg

      import core(false_elim, negation_elim);

      lemma explode(A C : Prop, x := A, nx := ¬A)
        ⊢ C;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``false_elim(C) then ⊢ False;`` reduces the goal to ``False``, which
      ``negation_elim(A)`` produces from ``x`` and ``nx`` (two ``cases``).

   **5c.** Prove ``¬¬A`` from ``A``.

   .. code-block:: alg

      import core(negation_intro, negation_elim);

      lemma dni(A : Prop, x := A)
        ⊢ ¬(¬A);
      proof
        by wip(?goal);
      wip;

   .. hint:: ``negation_intro(¬A) then P := ¬A ⊢ False;`` — now you hold ``P : ¬A``
      and ``x : A``, a contradiction, so ``negation_elim(A)`` gives ``False``.

6. Proof by contrapositive
==========================

**Goal:** prove ``P ⇒ Q``. **Method (classically):** prove ``¬Q ⇒ ¬P`` instead.

This one Algae's core **cannot** do. Turning a proof of ``¬Q ⇒ ¬P`` back into
``P ⇒ Q`` requires double-negation elimination — again, excluded middle — which
``core`` doesn't assume. (The *forward* direction is fine: from ``P ⇒ Q`` you can
always derive ``¬Q ⇒ ¬P``; it's the *reverse*, the one the technique relies on,
that's off-limits.) If you need classical reasoning, you'd add an excluded-middle
axiom of your own — but that's beyond this crash course, so there are no exercises
here.

7. Universal proof
==================

**Goal:** prove ``∀ x. P(x)``. **Method:** let ``x`` be *arbitrary* (a fresh
eigenvariable), then prove ``P(x)``. ``forall_intro`` hands you that fresh ``x`` in
its single premise; because ``x`` was arbitrary, proving ``P(x)`` proves it for
all.

.. admonition:: Exercises
   :class: tip

   **7a.** Everything equals itself: prove ``∀ x. x = x``.

   .. code-block:: alg

      import core(forall_intro, refl);

      sort T : Sort;

      lemma refl_all
        ⊢ ∀ (x : T) st x = x;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``forall_intro(T, _ = _) then x : T ⊢ x = x;`` carries a fresh ``x``
      in; close ``by refl(T, x)``.

   **7b.** Instantiate a universal: from ``∀ y. P(y)`` and a point ``a``, get
   ``P(a)``.

   .. code-block:: alg

      import core(forall_elim);

      sort T : Sort;

      lemma at_point(P : T → Prop, a : T, all := ∀ (y : T) st P(y))
        ⊢ P(a);
      proof
        by wip(?goal);
      wip;

   .. hint:: ``forall_elim(T, P, a) then ⊢ ∀ (y : T) st P(y); by all;``.

   **7c.** From ``∀ x. P(x)`` and ``∀ x. Q(x)``, prove ``∀ x. P(x) ∧ Q(x)``.

   .. code-block:: alg

      import core(forall_intro, forall_elim, and_intro);

      sort T : Sort;

      lemma forall_and(P Q : T → Prop, hp := ∀ (x : T) st P(x), hq := ∀ (x : T) st Q(x))
        ⊢ ∀ (x : T) st P(x) ∧ Q(x);
      proof
        by wip(?goal);
      wip;

   .. hint:: ``forall_intro`` for a fresh ``x``, then ``and_intro(P(x), Q(x))``;
      get each half by ``forall_elim`` on ``hp`` / ``hq`` at ``x``.

8. Existential proof (a witness)
================================

**Goal:** prove ``∃ x. P(x)``. **Method:** supply a specific *witness* ``a`` and
prove ``P(a)``. ``exists_intro`` takes the witness as an argument and leaves you
the single goal ``P(a)``.

.. admonition:: Exercises
   :class: tip

   **8a.** From a proof of ``P(a)``, conclude ``∃ x. P(x)``.

   .. code-block:: alg

      import core(exists_intro);

      sort T : Sort;

      lemma witnessed(P : T → Prop, a : T, pa := P(a))
        ⊢ ∃ (x : T) st P(x);
      proof
        by wip(?goal);
      wip;

   .. hint:: ``exists_intro(T, P, a) then ⊢ P(a); by pa;`` — you offered ``a`` as
      the witness, so the leftover goal is ``P`` at ``a``.

   **8b.** Something exists that equals ``a``: prove ``∃ x. x = a``.

   .. code-block:: alg

      import core(exists_intro, refl);

      sort T : Sort;

      lemma exists_eq(a : T)
        ⊢ ∃ (x : T) st x = a;
      proof
        by wip(?goal);
      wip;

   .. hint:: Use ``a`` itself as the witness: ``exists_intro(T, λ (z : T) st z = a,
      a) then ⊢ a = a;`` and close ``by refl(T, a)``.

   **8c.** Every element is *some* self-equal thing: prove ``∃ z. z = z``.

   .. code-block:: alg

      import core(exists_intro, refl);

      sort T : Sort;

      lemma exists_self(a : T)
        ⊢ ∃ (z : T) st z = z;
      proof
        by wip(?goal);
      wip;

   .. hint:: Witness with ``a``: ``exists_intro(T, λ (z : T) st z = z, a) then
      ⊢ a = a;`` then ``by refl(T, a)``.

9. Existential elimination (use a witness)
==========================================

**Goal:** conclude ``Q`` given ``∃ x. P(x)``. **Method:** introduce a *fresh*
witness ``a`` with ``P(a)`` in hand, and finish the proof using it — but ``Q``
must not mention ``a``, since you don't get to know which witness you were handed.
``exists_elim`` gives you that fresh ``x`` and the hypothesis ``witness := P(x)``.

.. admonition:: Exercises
   :class: tip

   **9a.** Unpack and immediately repack: from ``∃ x. P(x)``, prove ``∃ x. P(x)``.

   .. code-block:: alg

      import core(exists_intro, exists_elim);

      sort T : Sort;

      lemma repack(P : T → Prop, ex := ∃ (x : T) st P(x))
        ⊢ ∃ (x : T) st P(x);
      proof
        by wip(?goal);
      wip;

   .. hint:: ``exists_elim(T, P, ∃ (x : T) st P(x)) cases`` — feed ``ex`` for the
      existential, then in the witness branch ``x : T; witness := P(x)`` rebuild
      with ``exists_intro(T, P, x) then ⊢ P(x); by witness;``.

   **9b.** Flip an existential equation: from ``∃ x. x = a``, prove ``∃ y. a = y``.

   .. code-block:: alg

      import core(exists_intro, exists_elim, symmetry);

      sort T : Sort;

      lemma exists_flip(a : T, ex := ∃ (x : T) st x = a)
        ⊢ ∃ (y : T) st a = y;
      proof
        by wip(?goal);
      wip;

   .. hint:: Open ``ex`` with ``exists_elim``; in the witness branch you hold
      ``witness := x = a``. Offer ``x`` as the new witness
      (``exists_intro(T, λ (y : T) st a = y, x) then ⊢ a = x;``) and flip with
      ``symmetry(T, x, a) then ⊢ x = a; by witness;``.

   **9c.** Discharge under a universal: from ``∃ x. P(x)`` and
   ``∀ x. P(x) ⇒ R``, prove ``R``.

   .. code-block:: alg

      import core(exists_elim, forall_elim, implication_elim);

      sort T : Sort;

      lemma exists_use(P : T → Prop, R : Prop, ex := ∃ (x : T) st P(x),
                       use := ∀ (x : T) st P(x) ⇒ R)
        ⊢ R;
      proof
        by wip(?goal);
      wip;

   .. hint:: In the witness branch (``x : T; witness := P(x)``) instantiate ``use``
      at ``x`` with ``forall_elim`` to get ``P(x) ⇒ R``, then ``implication_elim``
      against ``witness``.

10. Equational reasoning (rewriting)
====================================

**Goal:** a goal built from an equation. **Method:** given ``a = b``, replace ``a``
with ``b`` (or ``b`` with ``a``) somewhere in the goal — that's ``forward`` /
``backward`` from :doc:`rewrite-reflexivity`, and ``refl`` closes anything of the
form ``x = x``.

.. admonition:: Exercises
   :class: tip

   **10a.** Close a definitional equation: prove ``0 + 0 = 0``.

   .. code-block:: alg

      import nat;

      lemma zero_plus_zero
        ⊢ 0 + 0 = 0;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``0 + 0 = 0`` is the axiom ``add_zero_left`` at ``0``: ``by
      add_zero_left(0);``.

   **10b.** Rewrite, then reflect: prove ``n = 0 + n``.

   .. code-block:: alg

      import nat;
      import core(refl, forward);

      lemma zero_left_flip(n : Nat)
        ⊢ n = 0 + n;
      proof
        by wip(?goal);
      wip;

   .. hint:: Turn the ``0 + n`` on the right into ``n`` with
      ``forward(Nat, 0 + n, n, add_zero_left(n), n = _) then ⊢ n = n;`` and close
      ``by refl(Nat, n)``.

   **10c.** Flip an equality: from ``a = b``, prove ``b = a``.

   .. code-block:: alg

      import nat;
      import core(symmetry);

      lemma flip_eq(a b : Nat, h := a = b)
        ⊢ b = a;
      proof
        by wip(?goal);
      wip;

   .. hint:: ``symmetry(Nat, a, b)`` concludes ``b = a`` from ``⊢ a = b``:
      ``then ⊢ a = b; by h;``.

That's the toolbox. Almost every proof in the standard library — and every
monster in the Dungeon Proof Crawler — is these moves, combined. When a goal
stumps you, ask which *shape* it has (an implication? a ``∀``? an equation?) and
reach for the matching technique.
