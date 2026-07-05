# One witness is all existence asks for.
import core(exists_intro);

sort T : Sort;

lemma banish(P : T -> Prop, a : T, pa := P(a))
  |- exists (x : T) st P(x);
proof
  by wip;
wip;
