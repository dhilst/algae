# Happy: result_cases over a Result(A, E), proving the trivial r = r.
import result(result_cases, refl);

lemma e(A E : Sort, r : Result(A, E))
  |- r = r;
proof
  by result_cases(A, E, r, _ = _) cases
    case
      x : A;
      |- ok(x) = ok(x);
      by refl(Result(A, E), ok(x));
    qed;
    case
      e : E;
      |- err(e) = err(e);
      by refl(Result(A, E), err(e));
    qed;
  qed;
qed;
