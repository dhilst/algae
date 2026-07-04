# Sad: forall_intro concludes `forall x. P(x)`, but the goal is `exists x. P(x)`.
# The rule conclusion does not match the goal, so the checker must reject this.
#expect: tactic `forall_intro`: rule conclusion does not match the current goal
import core(refl, forall_intro);

sort T : Sort;

lemma bad
  |- exists (x : T) st x = x;
proof
  by forall_intro(T, _ = _)
  then
    x : T;
    |- x = x;
    by refl(T, x);
qed;
