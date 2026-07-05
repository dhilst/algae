# Assume the forbidden, and let it collapse into falsehood.
import core(negation_intro, implication_elim);

lemma banish(A : Prop, f := A => False)
  |- ~A;
proof
  by wip;
wip;
