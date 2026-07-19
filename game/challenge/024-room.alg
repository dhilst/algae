sort T : Sort;

rule forall_elim(T : Sort, P : T → Prop, x : T)
  ⊢ ∀ (y : T) st P(y)
  ────────────────────────
  ⊢ P(x)
end;

lemma at_point(P : T → Prop, a : T, all := ∀ (y : T) st P(y))
  ⊢ P(a);
proof
  by wip(?goal);
wip;
