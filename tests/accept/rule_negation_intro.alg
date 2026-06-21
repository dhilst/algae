# Happy path for core rule `negation_intro` ((hyp P) |- False ==> |- ~P).
# The False subgoal is discharged by an inconsistent axiom (we test the rule's
# mechanics, not soundness).
import core(negation_intro);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom absurd
  |- False;

lemma neg
  |- ~(a = b);
proof
  by negation_intro(a = b)
  case
    hyp := a = b;
    |- False;
  proof
    by absurd;
  qed;
qed;
