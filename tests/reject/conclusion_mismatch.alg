# `refl` proves `z = z`, but the goal is `z = o`. With no equation
# relating z and o, the proof checker must reject this.
import core(refl);

sort Nat : Sort;

op z : -> Nat;
op o : -> Nat;

lemma bad
  |- z = o;
proof
  by refl(Nat, z);
qed;
