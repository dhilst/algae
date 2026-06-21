# Sad path: biconditional_elim_left(P, Q) concludes `P => Q` ((a=a) => (b=b)),
# but the goal is the reversed `Q => P` ((b=b) => (a=a)), so the rule conclusion
# cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(biconditional_elim_left);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom equiv
  |- (a = a) <=> (b = b);

lemma bad
  |- (b = b) => (a = a);
proof
  by biconditional_elim_left(a = a, b = b)
  case
    |- (a = a) <=> (b = b);
  proof
    by equiv;
  qed;
qed;
