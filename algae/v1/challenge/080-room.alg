# Induction over a list. The cons cell is yours to close.
import list(list_induction, refl);

lemma banish(A : Sort, xs : List(A))
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
      by wip;
    wip;
  wip;
wip;
