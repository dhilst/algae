# Happy path for adt rule `product_reflect_left`
# (|- P /\ Q ==> |- P). The conjunction premise is asserted by axiom.
import adt(product_reflect_left);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma left
  |- a = a;
proof
  by product_reflect_left(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
