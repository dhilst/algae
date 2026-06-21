# Happy: option_cases over an Option(A), proving the trivial m = m.
import option(option_cases, reflexivity);

lemma e(A : Sort, m : Option(A))
  |- m = m;
proof
  by option_cases(A, m, _ = _) cases
    case
      |- none = none;
    proof
      by reflexivity(None, none);
    qed;
    case
      x : A;
      |- some(x) = some(x);
    proof
      by reflexivity(Option(A), some(x));
    qed;
  qed;
qed;
