# What holds for all holds at any point.
import core(forall_elim);

sort T : Sort;

lemma banish(P : T -> Prop, a : T, all := forall (y : T) st P(y))
  |- P(a);
proof
  by forall_elim(T, P, a)
  then
    |- forall (y : T) st P(y);
    by wip;
wip;
