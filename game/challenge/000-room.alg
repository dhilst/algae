sort Nat : Sort;
op 0 : → Nat;
op s : Nat → Nat;
op + : Nat * Nat → Nat;
axiom add_zero_left(n : Nat)     ⊢ 0 + n = n;
axiom add_succ_left(n m : Nat)   ⊢ s(n) + m = s(n + m);

lemma zero_plus_zero
  ⊢ 0 + 0 = 0;
proof
  by wip(?goal);
wip;
