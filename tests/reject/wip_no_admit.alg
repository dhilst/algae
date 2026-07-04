# `wip` may only close a proof that actually admits a goal; a complete proof
# must use `qed`.
import core(refl);

sort Nat : Sort;
op z : -> Nat;

lemma t
  |- z = z;
proof
  by refl(Nat, z);
wip;
