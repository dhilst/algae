# Induction over the built-in naturals ℕ. `0` is the numeric literal; `s` is
# successor. This exercises rule declarations, propositions/sequents, and rule
# application with explicit cases. Nothing is proof-checked yet.
op s : ℕ → ℕ;

var m, n : ℕ;

axiom add_zero_left ⊢ 0 + m = m;
axiom add_succ_left ⊢ s(n) + m = s(n + m);

rule induction(x : ℕ, P : ℕ → Prop)
  ⊢ P(0)
  P(x) ⊢ P(s(x))
  ─────────────────────
  ⊢ ∀ (n : ℕ) st P(n)
end

lemma add_zero_right ⊢ n + 0 = n;
proof
  apply induction(n, λ (n : ℕ) => n + 0 = n);

  case [⊢ 0 + 0 = 0]
    0 + 0;
    = 0 by add_zero_left;
  qed;

  case [ih := n + 0 = n ⊢ s(n) + 0 = s(n)]
    s(n) + 0;
    = s(n + 0) by add_succ_left;
    = s(n) by ih;
  qed;
qed;
