sort Nat : Sort;
op 0 : → Nat;
op s : Nat → Nat;
op + : Nat * Nat → Nat;
axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);
sort T : Sort;

axiom refl(T : Sort, x : T)  ⊢ x = x;

rule forward(T : Sort, a b : T, eq := a = b, P : T → Prop)
  ⊢ P(b)
  ────────────────────────
  ⊢ P(a)
end;

rule exists_intro(T : Sort, P : T → Prop, x : T)
  ⊢ P(x)
  ────────────────────────
  ⊢ ∃ (x : T) st P(x)
end;

lemma zero_left_flip(n : Nat)
  ⊢ n = 0 + n;
proof
  by wip(?goal);
wip;

lemma exists_eq(a : T)
  ⊢ ∃ (x : T) st x = a;
proof
  by wip(?goal);
wip;
