# Sad path: negation_elim(P) concludes `False`, but the goal is `a = b`, so the
# rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(negation_elim);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom pos
  |- a = b;

axiom neg
  |- ~(a = b);

lemma bad
  |- a = b;
proof
  by negation_elim(a = b) cases
    case
      |- a = b;
      by pos;
    qed;
    case
      |- ~(a = b);
      by neg;
    qed;
  qed;
qed;
