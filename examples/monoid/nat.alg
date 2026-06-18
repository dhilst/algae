# Naturals with a left-identity equation, ready to instantiate the monoid.
sort Nat : Sort;

op z : → Nat;
op s : Nat → Nat;
op add : Nat × Nat → Nat;

eq add_zero_left(n : Nat)
  add(z, n) = n;
