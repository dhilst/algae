# Happy path for core rule `and_right` (|- P /\ Q ==> |- Q).
import core(and_right, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma right
  |- b = b;
proof
  by and_right(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
