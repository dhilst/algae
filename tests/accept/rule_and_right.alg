# Happy path for core rule `and_right` (|- P /\ Q ==> |- Q).
import core(and_right, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma right
  |- b = b;
proof
  by and_right(a = a, b = b)
  case
    |- a = a /\ b = b;
  proof
    by conj;
  qed;
qed;
