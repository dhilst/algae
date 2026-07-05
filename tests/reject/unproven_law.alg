# The model proves `l1` but omits `l2`, which is in the theory's law set.
import core(refl);

sort S : Sort;

op e : -> S;

theory T(
  X : Sort,
  c : X
) {
  law l1(x : X)
    |- x = x;

  law l2(x : X)
    |- c = c;
};

model M satisfies T(S, e) iff {
  law l1;
  proof
    by refl(S, x);
  end;
};
