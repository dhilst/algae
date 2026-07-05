# One witness is all existence asks for.
import core(refl, exists_intro);

sort T : Sort;
op a : -> T;

lemma banish
  |- exists (x : T) st x = x;
proof
  by wip;
wip;
