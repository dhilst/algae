# Sad path: sum_cases requires the second case to prove P(inr(y)), i.e. under
# the motive `_ = _` the goal `inr(y) = inr(y)`. Here the second case wrongly
# asserts the `inl` constructor again (`inl(y) = inl(y)`), which cannot match
# the rule's `inr` premise, so the checker must reject.
#expect: case goal does not match the generated subgoal
import adt(sum_cases, reflexivity);

lemma e(A B : Sort, s : Sum(A, B))
  |- s = s;
proof
  by sum_cases(A, B, s, _ = _) cases
    case
      x : A;
      |- inl(x) = inl(x);
    proof
      by reflexivity(Sum(A, B), inl(x));
    qed;
    case
      y : B;
      |- inl(y) = inl(y);
    proof
      by reflexivity(Sum(A, B), inl(y));
    qed;
  qed;
qed;
