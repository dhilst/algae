# Happy: list_induction over a List(A), proving the trivial xs = xs.
import list(list_induction, refl);

lemma e(A : Sort, xs : List(A))
  |- xs = xs;
proof
  by list_induction(A, xs, _ = _) cases
    case
      |- nil = nil;
      by refl(List(A), nil);
    qed;
    case
      x : A;
      rest : List(A);
      ih := rest = rest;
      |- cons(x, rest) = cons(x, rest);
      by refl(List(A), cons(x, rest));
    qed;
  qed;
qed;
