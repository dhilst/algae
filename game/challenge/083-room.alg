import core(exists_elim, forall_elim, implication_elim);

sort T : Sort;

lemma exists_use(P : T → Prop, R : Prop, ex := ∃ (x : T) st P(x), use := ∀ (x : T) st P(x) ⇒ R)
  ⊢ R;
proof
  by wip;
wip;
