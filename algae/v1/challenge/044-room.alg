# Whichever branch is true, swap the sides.
import core(or_elim, or_intro_left, or_intro_right);

lemma banish(A B : Prop, d := A \/ B)
  |- B \/ A;
proof
  by wip;
wip;
