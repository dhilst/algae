# Sad: list_induction `cons` case states the wrong goal (nil(A) = nil(A) instead
# of cons(A, x, rest) = cons(A, x, rest)).
#expect: case goal does not match the generated subgoal
import list(list_induction, refl);

lemma e(A : Sort, xs : List(A))
  |- xs = xs;
proof
  by list_induction(A, xs, _ = _) cases
    case
      |- nil(A) = nil(A);
      by refl(List(A), nil(A));
    qed;
    case
      x : A;
      rest : List(A);
      ih := rest = rest;
      |- nil(A) = nil(A);
      by refl(List(A), nil(A));
    qed;
  qed;
qed;
