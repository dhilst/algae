var n : ℕ;
rule induction(x : ℕ, P : ℕ → Prop)
  ⊢ P(0)
  ─────
  ⊢ ∀ (n : ℕ) st P(n)
