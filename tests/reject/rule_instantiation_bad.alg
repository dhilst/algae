# Sad: instantiation's conclusion is P(x) for the eigenvariable x. Here the
# eigenvariable argument is `c`, so the conclusion is `c = c`, but the goal is
# `x = x`. The rule conclusion does not match the goal, so it is rejected.
#expect: tactic `instantiation`: rule conclusion does not match the current goal
import core(instantiation);

sort T : Sort;
op c : -> T;
axiom all_eq |- forall (y : T) st y = y;

lemma bad(x : T)
  |- x = x;
proof
  by instantiation(T, _ = _, c)
  case
    |- forall (y : T) st y = y;
  proof
    by all_eq;
  qed;
qed;
