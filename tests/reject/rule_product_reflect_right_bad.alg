# Sad path: product_reflect_right(P, Q) concludes Q (the right conjunct), but the
# goal here is the left conjunct `a = a`. The conclusion cannot match the goal,
# so the checker must reject.
#expect: tactic `product_reflect_right`: rule conclusion does not match the current goal
import adt(product_reflect_right);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma bad
  |- a = a;
proof
  by product_reflect_right(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
