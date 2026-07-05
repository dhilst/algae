# Name no witness; prove it for an arbitrary one.
import core(refl, forall_intro);

sort T : Sort;

lemma banish
  |- forall (x : T) st x = x;
proof
  by forall_intro(T, _ = _)
  then
    x : T;
    |- x = x;
    by wip;
wip;
