# Happy path for adt rule `product_reflect_intro`
# (|- P ; |- Q ==> |- P /\ Q). Like and_intro: two cases, each conjunct by
# reflexivity.
import adt(product_reflect_intro, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma conj
  |- a = a /\ b = b;
proof
  by product_reflect_intro(a = a, b = b) cases
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
