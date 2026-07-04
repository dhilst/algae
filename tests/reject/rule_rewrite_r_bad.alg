# Sad: rewrite_r motive does not match the goal. With eq : a = b and motive
# `_ = c`, P(b) = (b = c), but the goal is `b = b`. The conclusion the rule
# produces (b = c) is not the goal, so the checker must reject this.
#expect: tactic `rewrite_r`: rule conclusion does not match the current goal
import core(refl, rewrite_r);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;

lemma bad
  |- b = b;
proof
  by rewrite_r(T, a, b, ab, _ = c)
  then
    |- a = c;
    by refl(T, a);
qed;
