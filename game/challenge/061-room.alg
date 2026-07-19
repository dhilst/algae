import core(forward, refl);

sort Nat : Sort;
op z : → Nat;
op s : Nat → Nat;
op add : Nat * Nat → Nat;

axiom add_zero(n : Nat)    ⊢ add(z, n) = n;
axiom add_succ(n m : Nat)  ⊢ add(s(n), m) = s(add(n, m));

lemma add_left_zero(n : Nat)
  ⊢ add(z, n) = n;
proof
  by wip(?goal);
wip;

lemma one_plus_zero
  ⊢ add(s(z), z) = s(z);
proof
  by wip(?goal);
wip;

lemma one_plus_one
  ⊢ add(s(z), s(z)) = s(s(z));
proof
  by wip(?goal);
wip;
