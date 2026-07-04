# Sad path: or_intro_left(P, Q) concludes `P \/ Q` (a=a \/ b=b), but the goal is
# the flipped `b=b \/ a=a`, so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(or_intro_left, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- b = b \/ a = a;
proof
  by or_intro_left(a = a, b = b)
  then
    |- a = a;
    by refl(T, a);
qed;
