# Assume the forbidden, and let it collapse into falsehood.
import core(negation_intro);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom absurd |- False;

lemma banish
  |- ~(a = b);
proof
  by negation_intro(a = b)
  then
    hyp := a = b;
    |- False;
    by wip;
wip;
