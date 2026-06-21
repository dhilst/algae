# Sad: list_induction `cons` case states the wrong goal (nil = nil instead of cons(x, rest) = cons(x, rest)).
#expect: case goal does not match the generated subgoal
import list(list_induction, reflexivity);

lemma e(A : Sort, xs : List(A))
  |- xs = xs;
proof
  by list_induction(A, xs, _ = _) cases
    case
      |- nil = nil;
    proof
      by reflexivity(List(A), nil);
    qed;
    case
      x : A;
      rest : List(A);
      ih := rest = rest;
      |- nil = nil;
    proof
      by reflexivity(List(A), nil);
    qed;
  qed;
qed;
