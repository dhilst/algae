# Sad path: sum_cases requires the second case to prove P(inr(A, B, y)), i.e.
# under the motive `_ = _` the goal `inr(A, B, y) = inr(A, B, y)`. Here the
# second case wrongly asserts the `inl` constructor again, which cannot match
# the rule's `inr` premise, so the checker must reject.
#expect: case goal does not match the generated subgoal
import adt(sum_cases, refl);

lemma e(A B : Sort, s : Sum(A, B))
  |- s = s;
proof
  by sum_cases(A, B, s, _ = _) cases
    case
      x : A;
      |- inl(A, B, x) = inl(A, B, x);
      by refl(Sum(A, B), inl(A, B, x));
    qed;
    case
      y : B;
      |- inl(A, B, y) = inl(A, B, y);
      by refl(Sum(A, B), inl(A, B, y));
    qed;
  qed;
qed;
