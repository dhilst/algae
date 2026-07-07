import core(exists_intro, exists_elim);

sort T : Sort;

lemma repack(P : T → Prop, ex := ∃ (x : T) st P(x))
  ⊢ ∃ (x : T) st P(x);
proof
  by wip(?goal);
wip;
