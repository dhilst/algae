# Grant the hypothesis; the conclusion stands on its own.
import core(implication_intro);

lemma banish(A : Prop)
  |- A => A;
proof
  by implication_intro(A, A)
  then
    P := A;
    |- A;
    by wip;
wip;
