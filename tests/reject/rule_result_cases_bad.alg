# Sad: result_cases `ok` case states the wrong goal (err(x) instead of ok(x)).
#expect: case goal does not match the generated subgoal
import result(result_cases, reflexivity);

lemma e(A E : Sort, r : Result(A, E))
  |- r = r;
proof
  by result_cases(A, E, r, _ = _) cases
    case
      x : A;
      |- err(x) = err(x);
    proof
      by reflexivity(Result(A, E), err(x));
    qed;
    case
      e : E;
      |- err(e) = err(e);
    proof
      by reflexivity(Result(A, E), err(e));
    qed;
  qed;
qed;
