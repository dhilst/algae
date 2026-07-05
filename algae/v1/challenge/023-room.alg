# What holds for all holds for you.
import core(forall_elim);

sort T : Sort;
axiom all_eq |- forall (y : T) st y = y;

lemma banish(x : T)
  |- x = x;
proof
  by forall_elim(T, _ = _, x)
  then
    |- forall (y : T) st y = y;
    by wip;
wip;
