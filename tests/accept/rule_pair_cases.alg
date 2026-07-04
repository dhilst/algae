# Happy path for adt rule `pair_cases`
# (x : A, y : B |- P(pair(x, y)) ==> |- P(p)).
# Splits p into pair(x, y); the motive P = (lambda k. k = k), so the premise
# goal is the trivial `pair(x, y) = pair(x, y)`, discharged by refl.
import adt(pair_cases, refl);

lemma e(A B : Sort, p : Pair(A, B))
  |- p = p;
proof
  by pair_cases(A, B, p, _ = _)
  then
    x : A;
    y : B;
    |- pair(x, y) = pair(x, y);
    by refl(Pair(A, B), pair(x, y));
qed;
