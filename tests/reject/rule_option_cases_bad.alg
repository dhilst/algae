# Sad: option_cases `some` case states the wrong goal (none = none instead of some(x) = some(x)).
#expect: case goal does not match the generated subgoal
import option(option_cases, refl);

lemma e(A : Sort, m : Option(A))
  |- m = m;
proof
  by option_cases(A, m, _ = _) cases
    case
      |- none = none;
    proof
      by refl(None, none);
    qed;
    case
      x : A;
      |- none = none;
    proof
      by refl(None, none);
    qed;
  qed;
qed;
