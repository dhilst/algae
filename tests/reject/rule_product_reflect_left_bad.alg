# Sad path: product_reflect_left(P, Q) concludes P (the left conjunct), but the
# goal here is the right conjunct `b = b`. The conclusion cannot match the goal,
# so the checker must reject.
#expect: tactic `product_reflect_left`: rule conclusion does not match the current goal
import adt(product_reflect_left);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma bad
  |- b = b;
proof
  by product_reflect_left(a = a, b = b)
  case
    |- a = a /\ b = b;
  proof
    by conj;
  qed;
qed;
