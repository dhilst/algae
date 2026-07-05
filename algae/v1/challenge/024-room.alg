# Both directions, and the gate swings free.
import core(biconditional_intro);

lemma banish(A B : Prop, fwd := A => B, bwd := B => A)
  |- A <=> B;
proof
  by biconditional_intro(A, B) cases
    case
      |- A => B;
      by wip;
    wip;
    case
      |- B => A;
      by wip;
    wip;
  wip;
wip;
