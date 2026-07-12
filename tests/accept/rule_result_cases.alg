# Happy: result_cases over a Result(A, E), proving the trivial r = r.
import result(result_cases, refl);

lemma e(A E : Sort, r : Result(A, E))
  |- r = r;
proof
  by result_cases(A, E, r, _ = _) cases
    case
      x : A;
      |- ok(A, E, x) = ok(A, E, x);
      by refl(Result(A, E), ok(A, E, x));
    qed;
    case
      e : E;
      |- err(A, E, e) = err(A, E, e);
      by refl(Result(A, E), err(A, E, e));
    qed;
  qed;
qed;
