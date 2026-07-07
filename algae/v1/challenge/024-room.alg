import core(forall_elim);

sort T : Sort;

lemma at_point(P : T → Prop, a : T, all := ∀ (y : T) st P(y))
  ⊢ P(a);
proof
  by wip;
wip;
