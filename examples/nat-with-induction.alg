# The naturals as a user-declared sort with z/successor, and induction as an
# explicit inference rule. add_zero_right is proved by induction: each `apply`
# step carries its cases, then `therefore <state>`, then its subproof
# terminator. Proofs are structure-checked; rewrite steps are not discharged.
sort Nat : Sort;

op z : → Nat;
op s : Nat → Nat;

op add : Nat × Nat → Nat;

eq add_zero_left(n : Nat)
  add(z, n) = n;

eq add_succ_left(n m : Nat)
  add(s(n), m) = s(add(n, m));

rule reflexivity(T : Sort, x : T)
  ─────────────────────────
  ⊢ x = x
end;

rule induction(P : Nat → Prop)
  case base
    ⊢ P(z)
  end;
  case step
    n : Nat, P(n) ⊢ P(s(n))
  end;
  ─────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;

lemma add_zero_right(n : Nat)
  add(n, z) = n;
proof
  goal
    ⊢ add(n, z) = n
  by apply induction(λ (n : Nat) => add(n, z) = n)
    case base
      goal
        ⊢ add(z, z) = z
      by rewrite > add_zero_left(z) with (add(z, z) := z)
      therefore
        ⊢ z = z;
      goal
        ⊢ z = z
      by apply reflexivity(Nat, z)
      therefore done
      qed;
    qed;
    case step
      goal
        n : Nat,
        ih := add(n, z) = n
        ⊢ add(s(n), z) = s(n)
      by rewrite > add_succ_left(n, z) with (add(s(n), z) := s(add(n, z)))
      therefore
        n : Nat,
        ih := add(n, z) = n
        ⊢ s(add(n, z)) = s(n);
      goal
        n : Nat,
        ih := add(n, z) = n
        ⊢ s(add(n, z)) = s(n)
      by rewrite > ih with (add(n, z) := n)
      therefore
        n : Nat,
        ih := add(n, z) = n
        ⊢ s(n) = s(n);
      goal
        n : Nat,
        ih := add(n, z) = n
        ⊢ s(n) = s(n)
      by apply reflexivity(Nat, s(n))
      therefore done
      qed;
    qed;
  therefore done
  qed;
qed;
