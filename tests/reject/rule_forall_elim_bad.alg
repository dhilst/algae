# Sad: forall_elim's conclusion is P(x) for the witness x. Here the witness
# argument is `c`, so the conclusion is `c = c`, but the goal is `x = x`. The
# rule conclusion does not match the goal, so it is rejected.
#expect: tactic `forall_elim`: rule conclusion does not match the current goal
import core(forall_elim);

sort T : Sort;
op c : -> T;
axiom all_eq |- forall (y : T) st y = y;

lemma bad(x : T)
  |- x = x;
proof
  by forall_elim(T, _ = _, c)
  then
    |- forall (y : T) st y = y;
    by all_eq;
qed;
