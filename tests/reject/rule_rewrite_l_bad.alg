# Sad: rewrite_l motive does not match the goal. With eq : a = b and motive
# `_ = c`, P(a) = (a = c), but the goal is `a = a`. The conclusion the rule
# produces (a = c) is not the goal, so the checker must reject this.
#expect: tactic `rewrite_l`: rule conclusion does not match the current goal
import core(reflexivity, rewrite_l);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;

lemma bad
  |- a = a;
proof
  by rewrite_l(T, a, b, ab, _ = c)
  case
    |- b = c;
  proof
    by reflexivity(T, b);
  qed;
qed;
