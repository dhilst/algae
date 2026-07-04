# Sad path: and_right(P, Q) concludes `Q` (= b = b), but the goal is `P` (a = a),
# so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(and_right);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom conj
  |- a = a /\ b = b;

lemma bad
  |- a = a;
proof
  by and_right(a = a, b = b)
  then
    |- a = a /\ b = b;
    by conj;
qed;
