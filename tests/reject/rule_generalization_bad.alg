# Sad: generalization concludes `forall x. P(x)`, but the goal is `exists x. P(x)`.
# The rule conclusion does not match the goal, so the checker must reject this.
#expect: tactic `generalization`: rule conclusion does not match the current goal
import core(reflexivity, generalization);

sort T : Sort;

lemma bad
  |- exists (x : T) st x = x;
proof
  by generalization(T, _ = _)
  case
    x : T;
    |- x = x;
  proof
    by reflexivity(T, x);
  qed;
qed;
