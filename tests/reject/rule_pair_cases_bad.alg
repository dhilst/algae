# Sad path: pair_cases splits p into pair(x, y), so the premise goal under the
# motive `_ = _` must be `pair(x, y) = pair(x, y)`. Here the case asserts the
# wrong goal `x = x` (the field, not the reconstructed pair), which cannot match
# the rule's premise, so the checker must reject.
#expect: case goal does not match the generated subgoal
import adt(pair_cases, refl);

lemma e(A B : Sort, p : Pair(A, B))
  |- p = p;
proof
  by pair_cases(A, B, p, _ = _)
  case
    x : A;
    y : B;
    |- x = x;
  proof
    by refl(A, x);
  qed;
qed;
