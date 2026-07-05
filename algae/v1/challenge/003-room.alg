# It offers you a pair and asks only for its right hand.
import core(and_right);

lemma banish(A B : Prop, both := A /\ B)
  |- B;
proof
  by and_right(A, B)
  then
    |- A /\ B;
    by wip;
wip;
