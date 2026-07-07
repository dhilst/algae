import core(forall_intro, forall_elim, and_intro);

sort T : Sort;

lemma forall_and(P Q : T → Prop, hp := ∀ (x : T) st P(x), hq := ∀ (x : T) st Q(x))
  ⊢ ∀ (x : T) st P(x) ∧ Q(x);
proof
  by wip(?goal);
wip;
