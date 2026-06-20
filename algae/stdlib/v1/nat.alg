import core(
  reflexivity,
  rewrite_r,
  transitivity
);

sort Nat : Sort;

op 0 : → Nat;

op s : Nat → Nat;

op + : Nat × Nat → Nat;

op × : Nat × Nat → Nat;

axiom add_zero_left(
  n : Nat
)
  ⊢ 0 + n = n;

axiom add_succ_left(
  n m : Nat
)
  ⊢ s(n) + m = s(n + m);

axiom mul_zero_left(
  n : Nat
)
  ⊢ 0 × n = 0;

axiom mul_succ_left(
  n m : Nat
)
  ⊢ s(n) × m = m + (n × m);

rule induction(
  P : Nat → Prop
)
  ⊢ P(0);
  n : Nat;
  ih := P(n);
  ⊢ P(s(n))
  ────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;

lemma add_zero_right
  ⊢ ∀ (n : Nat) st n + 0 = n;
proof
  by induction(
    _ + 0 = _
  ) cases
    case
      ⊢ 0 + 0 = 0;
    proof
      by add_zero_left(0);
    qed;

    case
      k : Nat;
      ih := k + 0 = k;
      ⊢ s(k) + 0 = s(k);
    proof
      by rewrite_r(
        Nat,
        k + 0,
        k,
        ih,
        s(k) + 0 = s(_)
      )
      case
        k : Nat;
        ih := k + 0 = k;
        ⊢ s(k) + 0 = s(k + 0);
      proof
        by add_succ_left(k, 0);
      qed;
    qed;
  qed;
qed;
