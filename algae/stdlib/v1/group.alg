import core(
  reflexivity,
  rewrite_r,
  transitivity,
  symmetry
);

theory Magma(
  S : Sort,
  mul : S × S → S
) laws
  law closure(
    x y : S
  )
    ⊢ mul(x, y) = mul(x, y);
qed;

theory Semigroup(
  S : Sort,
  mul : S × S → S
) laws
  include Magma(S, mul);

  law associativity(
    x y z : S
  )
    ⊢ mul(mul(x, y), z) = mul(x, mul(y, z));
qed;

theory Monoid(
  S : Sort,
  mul : S × S → S,
  e : S
) laws
  include Semigroup(S, mul);

  law left_identity(
    x : S
  )
    ⊢ mul(e, x) = x;

  law right_identity(
    x : S
  )
    ⊢ mul(x, e) = x;
qed;

theory CommutativeMonoid(
  S : Sort,
  mul : S × S → S,
  e : S
) laws
  include Monoid(S, mul, e);

  law commutativity(
    x y : S
  )
    ⊢ mul(x, y) = mul(y, x);
qed;

theory Group(
  S : Sort,
  mul : S × S → S,
  e : S,
  inv : S → S
) laws
  include Monoid(S, mul, e);

  law left_inverse(
    x : S
  )
    ⊢ mul(inv(x), x) = e;

  law right_inverse(
    x : S
  )
    ⊢ mul(x, inv(x)) = e;
qed;

theory AbelianGroup(
  S : Sort,
  mul : S × S → S,
  e : S,
  inv : S → S
) laws
  include Group(S, mul, e, inv);

  law commutativity(
    x y : S
  )
    ⊢ mul(x, y) = mul(y, x);
qed;
