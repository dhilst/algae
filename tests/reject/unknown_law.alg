# The model tries to prove `l_bogus`, which is not a law of the theory.
import core(reflexivity);

sort S : Sort;

op e : -> S;

theory T(
  X : Sort,
  c : X
) {
  law l1(x : X)
    |- x = x;
};

model M satisfies T(S, e) iff {
  law l1;
  proof
    by reflexivity(S, x);
  qed;

  law l_bogus;
  proof
    by reflexivity(S, x);
  qed;
};
