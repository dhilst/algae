# Something exists. Name it, then reach a truth that never needed the name.
import core(refl, exists_elim);

sort T : Sort;
op a : -> T;
axiom ex |- exists (x : T) st x = x;

lemma banish
  |- a = a;
proof
  by wip;
wip;
