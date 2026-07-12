# Happy: option_cases over an Option(A), proving the trivial m = m.
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
      |- some(A, x) = some(A, x);
      by refl(Option(A), some(A, x));
    qed;
  qed;
qed;
