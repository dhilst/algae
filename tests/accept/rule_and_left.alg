# Happy path for core rule `and_left` (|- P /\ Q ==> |- P).
import core(and_left, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma left
  |- a = a;
proof
  by and_left(a = a, b = b)
  case
    |- a = a /\ b = b;
  proof
    by conj;
  qed;
qed;
