# Happy path for core rule `transitivity` (|- x = y ; |- y = z ==> |- x = z).
import core(transitivity);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;
axiom bc |- b = c;

lemma trans
  |- a = c;
proof
  by transitivity(T, a, b, c) cases
    case
      |- a = b;
      by ab;
    qed;
    case
      |- b = c;
      by bc;
    qed;
  qed;
qed;
