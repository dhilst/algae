=============================
Proving with auxiliary lemmas
=============================

By now your proofs chain a fair number of ``by`` steps. Real developments get
bigger still — and the cure is the same one you'd reach for in any program: pull
a self-contained piece out, give it a name, and reuse it. In Algae that piece is
a **lemma**, and the payoff is direct:

  Once a lemma's ``qed`` checks, it becomes a *fact* — you invoke it by name with
  ``by``, exactly like an axiom or an inference rule.

Its parameters become the arguments you pass; its goal becomes the conclusion it
discharges. Because a proved lemma has nothing left to prove, applying it closes
the goal outright — zero subgoals, just like an axiom.

A proof becomes a fact
======================

Here's a small helper — conjunction is commutative — and a second lemma that
*uses* it:

.. code-block:: alg

   import core(and_intro, and_left, and_right, implication_intro);

   lemma and_comm(A B : Prop, both := A ∧ B)
     ⊢ B ∧ A;
   proof
     by and_intro(B, A) cases
       case ⊢ B; by and_right(A, B) then ⊢ A ∧ B; by both; qed;
       case ⊢ A; by and_left(A, B) then ⊢ A ∧ B; by both; qed;
     qed;
   qed;

   lemma and_comm_imp(A B : Prop)
     ⊢ (A ∧ B) ⇒ (B ∧ A);
   proof
     by implication_intro(A ∧ B, B ∧ A)
     then P := A ∧ B ⊢ B ∧ A;
     by and_comm(A, B, P);
   qed;

Look at the last line. ``and_comm``'s signature is ``(A B : Prop, both := A ∧ B)``,
so invoking it takes three arguments: the two propositions ``A`` and ``B``, and a
*proof* of ``A ∧ B`` — here the hypothesis ``P`` that ``implication_intro`` just
handed us. The lemma's conclusion, ``B ∧ A``, matches the goal, so ``by and_comm(A,
B, P)`` finishes the branch on its own. A term parameter takes a term; a
``:=`` parameter takes a proof; a proved lemma behaves like any other rule with
zero premises.

Prove once, reuse forever
=========================

The real reason to factor a lemma out is that some facts are *expensive* to
prove. The natural-number fact ``∀ n. n + 0 = n`` needs a full induction (you met
it in :doc:`induction`); it lives, already proved, in |nat.alg| as
``add_zero_right``. You never want to redo that induction. Instead, ``import`` it
and instantiate it at whatever point you need:

.. code-block:: alg

   import nat;
   import core(forall_elim);

   lemma add_zero_at(a : Nat)
     ⊢ a + 0 = a;
   proof
     by forall_elim(Nat, _ + 0 = _, a)
     then ⊢ ∀ (n : Nat) st n + 0 = n;
     by add_zero_right;
   qed;

``add_zero_right`` takes no arguments — it's a closed universal fact — so ``by
add_zero_right;`` discharges the ``∀`` goal directly, and ``forall_elim`` peels it
down to the instance ``a + 0 = a``. The whole stdlib is built this way: each
module's harder theorems lean on the simpler lemmas below them.

.. note::

   Order doesn't matter *within* a unit. The checker reads every declaration in
   the file before it checks any proof, so a lemma may cite one defined further
   down (or a mutually-useful pair may cite each other). Idiomatic style is still
   bottom-up — helpers first — because it reads like the dependency order.

.. admonition:: Your turn
   :class: tip

   Disjunction is commutative too — and it's already proved for you below as
   ``or_comm``. Use it to prove that ``A ∨ B`` *implies* ``B ∨ A``.

   .. code-block:: alg

      import core(or_elim, or_intro_left, or_intro_right, implication_intro);

      lemma or_comm(A B : Prop, d := A ∨ B)
        ⊢ B ∨ A;
      proof
        by or_elim(A, B, B ∨ A) cases
          case ⊢ A ∨ B; by d; qed;
          case P := A ⊢ B ∨ A; by or_intro_right(B, A) then ⊢ A; by P; qed;
          case Q := B ⊢ B ∨ A; by or_intro_left(B, A) then ⊢ B; by Q; qed;
        qed;
      qed;

      lemma or_comm_imp(A B : Prop)
        ⊢ (A ∨ B) ⇒ (B ∨ A);
      proof
        by wip(?goal);
      wip;

   .. hint::

      Mirror ``and_comm_imp``. Start with ``by implication_intro(A ∨ B, B ∨ A)``,
      which leaves ``then P := A ∨ B ⊢ B ∨ A;``. Now ``or_comm``'s conclusion is
      exactly ``B ∨ A`` — feed it the two propositions and your hypothesis:
      ``by or_comm(A, B, P);``.

That's the whole trick. A lemma is a proof you get to name and a fact you get to
reuse — the same duality that makes the standard library, and any development you
build on top of it, scale.
