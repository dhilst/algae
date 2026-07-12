# Happy path for adt rule `sum_cases`
# (x : A |- P(inl(A, B, x)) ; y : B |- P(inr(A, B, y)) ==> |- P(s)).
# Motive P = (lambda k. k = k), so each premise goal is trivial by refl.
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
      |- inr(A, B, y) = inr(A, B, y);
      by refl(Sum(A, B), inr(A, B, y));
    qed;
  qed;
qed;
