# The symbol `w` is never declared: elaboration must reject this.
sort Nat : Sort;

op z : -> Nat;

axiom bad
  |- z = w;
