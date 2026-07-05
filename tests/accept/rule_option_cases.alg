# Happy: option_cases over an Option(A), proving the trivial m = m.
import option(option_cases, refl);

lemma e(A : Sort, m : Option(A))
  |- m = m;
proof
  by option_cases(A, m, _ = _) cases
    case
      |- none = none;
      by refl(None, none);
    qed;
    case
      x : A;
      |- some(x) = some(x);
      by refl(Option(A), some(x));
    qed;
  qed;
qed;
