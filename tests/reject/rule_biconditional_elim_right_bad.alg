# Sad path: biconditional_elim_right(P, Q) concludes `Q => P` ((b=b) => (a=a)),
# but the goal is the reversed `P => Q` ((a=a) => (b=b)), so the rule conclusion
# cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(biconditional_elim_right);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom equiv
  |- (a = a) <=> (b = b);

lemma bad
  |- (a = a) => (b = b);
proof
  by biconditional_elim_right(a = a, b = b)
  case
    |- (a = a) <=> (b = b);
  proof
    by equiv;
  qed;
qed;
