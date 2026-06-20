# Exercises `generalization` (∀-introduction with an eigenvariable) and that
# its premise variable comes from the context, not a rule parameter.
import core(reflexivity, generalization);

sort Nat : Sort;

lemma all_self_eq
  |- forall (n : Nat) st n = n;
proof
  by generalization(Nat, lambda (k : Nat) st k = k)
  case
    n : Nat;
    |- n = n;
  proof
    by reflexivity(Nat, n);
  qed;
qed;
