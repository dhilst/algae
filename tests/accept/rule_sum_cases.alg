# Happy path for adt rule `sum_cases`
# (x : A |- P(inl(x)) ; y : B |- P(inr(y)) ==> |- P(s)).
# Motive P = (lambda k. k = k), so each premise goal is trivial by refl.
import adt(sum_cases, refl);

lemma e(A B : Sort, s : Sum(A, B))
  |- s = s;
proof
  by sum_cases(A, B, s, _ = _) cases
    case
      x : A;
      |- inl(x) = inl(x);
    proof
      by refl(Sum(A, B), inl(x));
    qed;
    case
      y : B;
      |- inr(y) = inr(y);
    proof
      by refl(Sum(A, B), inr(y));
    qed;
  qed;
qed;
