# The demon holds the ring. Prove n + 0 = n for every number under the earth.
# Two rites remain unspoken: the base, and the closing of the successor.
import nat(induction, add_zero_left, add_succ_left);
import core(rewrite_r);

lemma banish
  |- forall (n : Nat) st n + 0 = n;
proof
  by induction(_ + 0 = _) cases
    case
      |- 0 + 0 = 0;
      by wip;
    wip;
    case
      k : Nat;
      ih := k + 0 = k;
      |- s(k) + 0 = s(k);
      by rewrite_r(Nat, k + 0, k, ih, s(k) + 0 = s(_))
      then |- s(k) + 0 = s(k + 0);
      by wip;
    wip;
  wip;
wip;
