# Sad path: negation_intro(P) concludes `~P` (~(a=b)), but the goal is the bare
# `a = b`, so the rule conclusion cannot match the goal.
#expect: rule conclusion does not match the current goal
import core(negation_intro);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom absurd
  |- False;

lemma bad
  |- a = b;
proof
  by negation_intro(a = b)
  case
    hyp := a = b;
    |- False;
  proof
    by absurd;
  qed;
qed;
