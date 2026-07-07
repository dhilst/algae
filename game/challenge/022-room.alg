sort T : Sort;

rule forall_intro(T : Sort, P : T → Prop)
  x : T ⊢ P(x)
  ────────────────────────
  ⊢ ∀ (x : T) st P(x)
end;

axiom refl(T : Sort, x : T)  ⊢ x = x;

lemma refl_all
  ⊢ ∀ (x : T) st x = x;
proof
  by wip(?goal);
wip;
