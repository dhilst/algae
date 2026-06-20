# `reflexivity` takes two arguments (T, x); supplying one must be rejected.
import core(reflexivity);

sort Nat : Sort;

op z : -> Nat;

lemma bad
  |- z = z;
proof
  by reflexivity(Nat);
qed;
