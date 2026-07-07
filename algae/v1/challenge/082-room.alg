import core(exists_intro, exists_elim, symmetry);

sort T : Sort;

lemma exists_flip(a : T, ex := ∃ (x : T) st x = a)
  ⊢ ∃ (y : T) st a = y;
proof
  by wip(?goal);
wip;
