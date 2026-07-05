# Something exists. Unpack it, and pack it up again.
import core(exists_intro, exists_elim);

sort T : Sort;

lemma banish(P : T -> Prop, ex := exists (x : T) st P(x))
  |- exists (x : T) st P(x);
proof
  by wip;
wip;
