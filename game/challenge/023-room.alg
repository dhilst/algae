import core(exists_intro);

sort T : Sort;

lemma witnessed(P : T → Prop, a : T, pa := P(a))
  ⊢ ∃ (x : T) st P(x);
proof
  by wip(?goal);
wip;
