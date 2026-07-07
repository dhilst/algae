sort T : Sort;

rule exists_intro(T : Sort, P : T → Prop, x : T)
  ⊢ P(x)
  ────────────────────────
  ⊢ ∃ (x : T) st P(x)
end;

lemma witnessed(P : T → Prop, a : T, pa := P(a))
  ⊢ ∃ (x : T) st P(x);
proof
  by wip(?goal);
wip;
