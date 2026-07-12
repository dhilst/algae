# Sad: result_cases `ok` case states the wrong goal (err instead of ok).
#expect: case goal does not match the generated subgoal
import result(result_cases, refl);

lemma e(A E : Sort, r : Result(A, E))
  |- r = r;
proof
  by result_cases(A, E, r, _ = _) cases
    case
      x : A;
      |- err(A, E, x) = err(A, E, x);
      by refl(Result(A, E), err(A, E, x));
    qed;
    case
      e : E;
      |- err(A, E, e) = err(A, E, e);
      by refl(Result(A, E), err(A, E, e));
    qed;
  qed;
qed;
