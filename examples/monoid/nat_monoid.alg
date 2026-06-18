# Instantiate the monoid at the naturals: T := Nat, unit := z, mul := add.
# Each prop becomes an obligation discharged by a named case.
include nat;
open nat (z, s, add);

include monoid with (
  T := nat::Nat,
  unit := z,
  mul := add
) props
  case left_identity
    goal
      ⊢ add(z, x) = x

    by rewrite >
         add_zero_left(x)
    with (
      add(z, x) := x
    )

    therefore
      done;
  qed;

  case associativity
    # associativity of add is not proved here; the case shape is required.
  qed;
qed;
