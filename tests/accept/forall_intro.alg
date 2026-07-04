# Exercises `forall_intro` (∀-introduction with an eigenvariable) and that
# its premise variable comes from the context, not a rule parameter.
import core(refl, forall_intro);

sort Nat : Sort;

lemma all_self_eq
  |- forall (n : Nat) st n = n;
proof
  by forall_intro(Nat, lambda (k : Nat) st k = k)
  case
    n : Nat;
    |- n = n;
  proof
    by refl(Nat, n);
  qed;
qed;
