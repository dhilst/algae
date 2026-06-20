# A proof that admits a goal with `by wip` must be closed by `wip`, not `qed`.
import core(reflexivity);

sort Nat : Sort;
op z : -> Nat;

lemma t
  |- z = z;
proof
  by wip;
qed;
