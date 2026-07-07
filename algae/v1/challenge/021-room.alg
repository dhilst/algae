sort Nat : Sort;
op 0 : → Nat;
op s : Nat → Nat;
op + : Nat * Nat → Nat;
axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);

rule symmetry(T : Sort, x y : T)
  ⊢ x = y
  ────────────────────────
  ⊢ y = x
end;

lemma flip_eq(a b : Nat, h := a = b)
  ⊢ b = a;
proof
  by wip(?goal);
wip;
