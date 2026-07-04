import core(
  refl,
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
  n : Nat, ih := P(n) ⊢ P(s(n))
  ──────────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;

lemma add_zero_right
  ⊢ ∀ (n : Nat) st n + 0 = n;     # what we want to prove
proof
  by induction(
    _ + 0 = _                     # start by doing induction at n
  ) cases
    # The base case
    case
      ⊢ 0 + 0 = 0;                # the current goal
    proof
      by add_zero_left(0);        # by def of add_zero_left(n) := 0 + n = n;
    qed;

    # The step case
    case
      k : Nat;                    # the context
      ih := k + 0 = k;            # induction hypothesis
      ⊢ s(k) + 0 = s(k);          # the current goal
    proof
      by rewrite_r(               # rewrite at right
        Nat,                      # the type of each side
        k + 0, k,                 # replace `k + 0` → `k`
        ih,                       # by the induction hypothesis (k + 0 = k)
        s(k) + 0 = s(_)           # at _, i.e. `s(k) + 0 = s(k)` → `s(k) + 0 = s(k + 0)`
      )
      then ⊢ s(k) + 0 = s(k + 0);   # new goal after the rewrite
      by add_succ_left(k, 0);       # by def: add_succ_left(n, m) := s(n) + m = s(n + m);
    qed;
  qed;
qed;
