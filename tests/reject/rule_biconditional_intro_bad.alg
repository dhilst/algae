# Sad path: biconditional_intro(P, Q) concludes `P <=> Q`, but the goal is the
# implication `(a=a) => (b=b)`, so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(biconditional_intro);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom fwd
  |- (a = a) => (b = b);

axiom bwd
  |- (b = b) => (a = a);

lemma bad
  |- (a = a) => (b = b);
proof
  by biconditional_intro(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
      by fwd;
    qed;
    case
      |- (b = b) => (a = a);
      by bwd;
    qed;
  qed;
qed;
