# Sad path: and_left(P, Q) concludes `P` (= a = a), but the goal is `Q` (b = b),
# so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(and_left);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma bad
  |- b = b;
proof
  by and_left(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
