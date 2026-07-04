# Happy path for adt rule `product_reflect_right`
# (|- P /\ Q ==> |- Q). The conjunction premise is asserted by axiom.
import adt(product_reflect_right);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma right
  |- b = b;
proof
  by product_reflect_right(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
