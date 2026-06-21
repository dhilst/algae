# Sad: induction base case states the wrong goal (s(0) = s(0) instead of 0 = 0).
#expect: case goal does not match the generated subgoal
import nat(induction, reflexivity);

lemma e
  |- forall (n : Nat) st n = n;
proof
  by induction(_ = _) cases
    case
      |- s(0) = s(0);
    proof
      by reflexivity(Nat, s(0));
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
