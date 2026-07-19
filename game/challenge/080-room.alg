sort T : Sort;

rule forall_intro(T : Sort, P : T → Prop)
  x : T ⊢ P(x)
  ────────────────────────
  ⊢ ∀ (x : T) st P(x)
end;

rule forall_elim(T : Sort, P : T → Prop, x : T)
  ⊢ ∀ (y : T) st P(y)
  ────────────────────────
  ⊢ P(x)
end;

rule and_intro(P Q : Prop)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

rule and_left(P Q : Prop)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

lemma forall_and(P Q : T → Prop, hp := ∀ (x : T) st P(x), hq := ∀ (x : T) st Q(x))
  ⊢ ∀ (x : T) st P(x) ∧ Q(x);
proof
  by wip(?goal);
wip;

lemma forall_and_at(P Q : T → Prop, hp := ∀ (x : T) st P(x), hq := ∀ (x : T) st Q(x), a : T)
  ⊢ P(a) ∧ Q(a);
proof
  by wip(?goal);
wip;

lemma forall_and_left(P Q : T → Prop, hp := ∀ (x : T) st P(x), hq := ∀ (x : T) st Q(x), a : T)
  ⊢ P(a);
proof
  by wip(?goal);
wip;
