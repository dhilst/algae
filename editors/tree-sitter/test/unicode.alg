# Unicode operator forms (spec section 2.6) must parse identically to ASCII.

import core;

sort Nat : Sort;
sort Result : Sort * Sort -> Sort;

op + : Nat * Nat -> Nat;
op may_fail : A -> B | Err;

axiom demo(
  P Q : Prop,
  x y : Nat
)
  ⊢ ¬P ∧ Q ∨ x = y ⇒ y = x ⇔ P;

rule sep_box(
  P : Prop
)
  ⊢ P
  ────────────────────────
  ⊢ P
end;

lemma quantified
  ⊢ forall (n : Nat) st exists (m : Nat) st n = m;
proof
  by demo(P, Q, x, y);
qed;

lemma lambda_uni
  ⊢ forall (n : Nat) st n = n;
proof
  by foo(λ (a : Nat) st a + 0 = a);
qed;
