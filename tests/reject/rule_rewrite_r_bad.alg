# Sad: backward motive does not match the goal. With eq : a = b and motive
# `_ = c`, P(b) = (b = c), but the goal is `b = b`. The conclusion the rule
# produces (b = c) is not the goal, so the checker must reject this.
#expect: tactic `backward`: rule conclusion does not match the current goal
import core(refl, backward);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;

lemma bad
  |- b = b;
proof
  by backward(T, a, b, ab, _ = c)
  then
    |- a = c;
    by refl(T, a);
qed;
