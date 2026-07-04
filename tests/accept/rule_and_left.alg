# Happy path for core rule `and_left` (|- P /\ Q ==> |- P).
import core(and_left, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma left
  |- a = a;
proof
  by and_left(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
