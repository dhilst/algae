# Happy path for adt rule `product_reflect_intro`
# (|- P ; |- Q ==> |- P /\ Q). Like and_intro: two cases, each conjunct by
# refl.
import adt(product_reflect_intro, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma conj
  |- a = a /\ b = b;
proof
  by product_reflect_intro(a = a, b = b) cases
    case
      |- a = a;
      by refl(T, a);
    qed;
    case
      |- b = b;
      by refl(T, b);
    qed;
  qed;
qed;
