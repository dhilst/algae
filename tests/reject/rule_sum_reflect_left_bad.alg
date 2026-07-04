# Sad path: sum_reflect_left(P, Q) concludes `P \/ Q`, but the goal here has the
# disjuncts swapped (`b = b \/ a = a`). The conclusion cannot match the goal,
# so the checker must reject.
#expect: tactic `sum_reflect_left`: rule conclusion does not match the current goal
import adt(sum_reflect_left, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- b = b \/ a = a;
proof
  by sum_reflect_left(a = a, b = b)
  then
    |- a = a;
    by refl(T, a);
qed;
