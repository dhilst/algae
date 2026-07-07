sort T : Sort;

rule forall_elim(T : Sort, P : T → Prop)
  ⊢ ∀ (y : T) st P(y)
  ────────────────────────
  x : T ⊢ P(x)
end;

rule exists_intro(T : Sort, P : T → Prop, x : T)
  ⊢ P(x)
  ────────────────────────
  ⊢ ∃ (x : T) st P(x)
end;

rule and_intro(P Q : Prop)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

lemma inst(P : T → Prop, all := ∀ (y : T) st P(y), a : T)
  ⊢ P(a);
proof
  by wip(?goal);
wip;

lemma all_to_ex(P : T → Prop, all := ∀ (y : T) st P(y), a : T)
  ⊢ ∃ (x : T) st P(x);
proof
  by wip(?goal);
wip;

lemma all_and_ex(P : T → Prop, all := ∀ (y : T) st P(y), a : T)
  ⊢ P(a) ∧ (∃ (x : T) st P(x));
proof
  by wip(?goal);
wip;
