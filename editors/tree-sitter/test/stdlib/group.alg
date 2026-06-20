import core(
  reflexivity,
  rewrite_r,
  transitivity,
  symmetry
);

theory Magma(
  S : Sort,
  op : S * S -> S
) {
  law closure(
    x y : S
  )
    |- op(x, y) = op(x, y);
};

theory Semigroup(
  S : Sort,
  op : S * S -> S
) {
  include Magma(S, op);

  law associativity(
    x y z : S
  )
    |- op(op(x, y), z) = op(x, op(y, z));
};

theory Monoid(
  S : Sort,
  op : S * S -> S,
  e : S
) {
  include Semigroup(S, op);

  law left_identity(
    x : S
  )
    |- op(e, x) = x;

  law right_identity(
    x : S
  )
    |- op(x, e) = x;
};

theory CommutativeMonoid(
  S : Sort,
  op : S * S -> S,
  e : S
) {
  include Monoid(S, op, e);

  law commutativity(
    x y : S
  )
    |- op(x, y) = op(y, x);
};

theory Group(
  S : Sort,
  op : S * S -> S,
  e : S,
  inv : S -> S
) {
  include Monoid(S, op, e);

  law left_inverse(
    x : S
  )
    |- op(inv(x), x) = e;

  law right_inverse(
    x : S
  )
    |- op(x, inv(x)) = e;
};

theory AbelianGroup(
  S : Sort,
  op : S * S -> S,
  e : S,
  inv : S -> S
) {
  include Group(S, op, e, inv);

  law commutativity(
    x y : S
  )
    |- op(x, y) = op(y, x);
};
