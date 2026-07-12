# Happy path for adt rule `pair_cases`
# (x : A, y : B |- P(pair(A, B, x, y)) ==> |- P(p)).
# Splits p into pair(A, B, x, y); the motive P = (lambda k. k = k), so the
# premise goal is the trivial `pair(A, B, x, y) = pair(A, B, x, y)`, by refl.
import adt(pair_cases, refl);

lemma e(A B : Sort, p : Pair(A, B))
  |- p = p;
proof
  by pair_cases(A, B, p, _ = _)
  then
    x : A;
    y : B;
    |- pair(A, B, x, y) = pair(A, B, x, y);
    by refl(Pair(A, B), pair(A, B, x, y));
qed;
