# Sad: exists_intro concludes an existential `exists (x : T) st P(x)`. Applied
# here to a plain equation goal (not existential), the rule's conclusion cannot
# match the goal, so `exists_intro` itself rejects the step.
#expect: tactic `exists_intro`: rule conclusion does not match
import core(refl, exists_intro);

sort T : Sort;
op a : -> T;

lemma bad
  |- a = a;
proof
  by exists_intro(T, _ = _, a)
  case
    |- a = a;
  proof
    by refl(T, a);
  qed;
qed;
