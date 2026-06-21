# Happy path for core rule `and_intro` (|- P ; |- Q ==> |- P /\ Q).
import core(and_intro, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma conj
  |- a = a /\ b = b;
proof
  by and_intro(a = a, b = b) cases
    case
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
    case
      |- b = b;
    proof
      by reflexivity(T, b);
    qed;
  qed;
qed;
