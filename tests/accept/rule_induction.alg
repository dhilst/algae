# Happy: induction over Nat, proving the trivial forall n. n = n.
import nat(induction, refl);

lemma e
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
      by refl(Nat, s(n));
    qed;
  qed;
qed;
