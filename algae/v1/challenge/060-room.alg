# The dragon coils around the induction. Take the successor step.
import nat(induction, refl);

lemma banish
  |- forall (n : Nat) st n = n;
proof
  by induction(_ = _) cases
    case
      |- 0 = 0;
      by refl(Nat, 0);
    qed;
    case
      n : Nat;
      ih := n = n;
      |- s(n) = s(n);
      by wip;
    wip;
  wip;
wip;
