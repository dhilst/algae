# The right side will do.
import core(or_intro_right);

lemma banish(A B : Prop, y := B)
  |- A \/ B;
proof
  by or_intro_right(A, B)
  then
    |- B;
    by wip;
wip;
