# Sad path: implication_elim(P, Q) concludes `Q` (b=b), but the goal is `a=a`,
# so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(implication_elim);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom imp
  |- (a = a) => (b = b);

axiom ante
  |- a = a;

lemma bad
  |- a = a;
proof
  by implication_elim(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
    proof
      by imp;
    qed;
    case
      |- a = a;
    proof
      by ante;
    qed;
  qed;
qed;
