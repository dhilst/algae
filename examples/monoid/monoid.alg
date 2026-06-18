# An abstract monoid: a carrier T, a unit, and an associative binary `mul`
# with the unit as left identity. The props are proof obligations any
# instantiation must discharge.
param T : Sort;

op unit : → T;
op mul : T × T → T;

prop left_identity(x : T)
  mul(unit, x) = x;

prop associativity(x y z : T)
  mul(mul(x, y), z) = mul(x, mul(y, z));
