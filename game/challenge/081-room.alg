# n + 0 = n. The rewrite is laid out; supply the defining law.
import nat(induction, add_zero_left, add_succ_left);
import core(rewrite_r);

lemma banish
  |- forall (n : Nat) st n + 0 = n;
proof
  by wip;
wip;
