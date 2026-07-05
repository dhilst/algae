# Two halves must both be spoken.
import core(and_intro);

lemma banish(A B : Prop, x := A, y := B)
  |- A /\ B;
proof
  by and_intro(A, B) cases
    case
      |- A;
      by wip;
    wip;
    case
      |- B;
      by wip;
    wip;
  wip;
wip;
