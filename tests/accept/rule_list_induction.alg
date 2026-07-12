# Happy: list_induction over a List(A), proving the trivial xs = xs.
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
      |- cons(A, x, rest) = cons(A, x, rest);
      by refl(List(A), cons(A, x, rest));
    qed;
  qed;
qed;
