# Sad: option_cases `some` case states the wrong goal (none(A) = none(A) instead
# of some(A, x) = some(A, x)).
#expect: case goal does not match the generated subgoal
import option(option_cases, refl);

lemma e(A : Sort, m : Option(A))
  |- m = m;
proof
  by option_cases(A, m, _ = _) cases
    case
      |- none(A) = none(A);
      by refl(Option(A), none(A));
    qed;
    case
      x : A;
      |- none(A) = none(A);
      by refl(Option(A), none(A));
    qed;
  qed;
qed;
