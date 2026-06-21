# Sad path: product_reflect_intro concludes `P /\ Q`, but the goal here is the
# bare `a = a`, so the rule conclusion cannot match the goal and the checker
# must reject.
#expect: tactic `product_reflect_intro`: rule conclusion does not match the current goal
import adt(product_reflect_intro, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- a = a;
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
