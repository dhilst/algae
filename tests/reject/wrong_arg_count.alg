# `refl` takes two arguments (T, x); supplying one must be rejected.
import core(refl);

sort Nat : Sort;

op z : -> Nat;

lemma bad
  |- z = z;
proof
  by refl(Nat);
qed;
