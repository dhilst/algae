# Name no witness; prove it for an arbitrary one.
import core(refl, forall_intro);

sort T : Sort;

lemma banish
  |- forall (x : T) st x = x;
proof
  by wip;
wip;
