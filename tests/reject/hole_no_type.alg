# `_` is only valid as a tactic predicate argument; a bare hole in a statement
# cannot be elaborated.
sort Nat : Sort;
op z : -> Nat;

axiom bad
  |- _ = z;
