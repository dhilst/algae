# Happy: induction over Nat, proving the trivial forall n. n = n.
import nat(induction, reflexivity);

lemma e
  |- forall (n : Nat) st n = n;
proof
  by induction(_ = _) cases
    case
      |- 0 = 0;
    proof
      by reflexivity(Nat, 0);
    qed;
    case
      n : Nat;
      ih := n = n;
      |- s(n) = s(n);
    proof
      by reflexivity(Nat, s(n));
    qed;
  qed;
qed;
