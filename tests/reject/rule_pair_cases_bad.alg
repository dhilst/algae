# Sad path: pair_cases splits p into pair(A, B, x, y), so the premise goal under
# the motive `_ = _` must be `pair(A, B, x, y) = pair(A, B, x, y)`. Here the case
# asserts the wrong goal `x = x` (the field, not the reconstructed pair), which
# cannot match the rule's premise, so the checker must reject.
#expect: `then` goal does not match the remaining subgoal
import adt(pair_cases, refl);

lemma e(A B : Sort, p : Pair(A, B))
  |- p = p;
proof
  by pair_cases(A, B, p, _ = _)
  then
    x : A;
    y : B;
    |- x = x;
    by refl(A, x);
qed;
