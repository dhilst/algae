# One truth is enough to tip the balance.
import core(or_intro_left);

lemma banish(A B : Prop, x := A)
  |- A \/ B;
proof
  by or_intro_left(A, B)
  then
    |- A;
    by wip;
wip;
