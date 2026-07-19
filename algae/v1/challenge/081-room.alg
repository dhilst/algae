sort T : Sort;

rule exists_intro(T : Sort, P : T → Prop, x : T)
  ⊢ P(x)
  ────────────────────────
  ⊢ ∃ (x : T) st P(x)
end;

rule exists_elim(T : Sort, P : T → Prop, Q : Prop)
  ⊢ ∃ (x : T) st P(x);
  x : T, witness := P(x) ⊢ Q
  ────────────────────────
  ⊢ Q
end;

rule forall_elim(T : Sort, P : T → Prop, x : T)
  ⊢ ∀ (y : T) st P(y)
  ────────────────────────
  ⊢ P(x)
end;

rule implication_elim(P Q : Prop)
  ⊢ P ⇒ Q;
  ⊢ P
  ────────────────────────
  ⊢ Q
end;

lemma mp(A B : Prop, f := A ⇒ B, x := A)
  ⊢ B;
proof
  by wip(?goal);
wip;

lemma at_imp(P Q : T → Prop, f := ∀ (x : T) st P(x) ⇒ Q(x), a : T)
  ⊢ P(a) ⇒ Q(a);
proof
  by wip(?goal);
wip;

lemma ex_map(P Q : T → Prop, ex := ∃ (x : T) st P(x), f := ∀ (x : T) st P(x) ⇒ Q(x))
  ⊢ ∃ (x : T) st Q(x);
proof
  by wip(?goal);
wip;
