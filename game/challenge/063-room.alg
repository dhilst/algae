rule and_intro(P Q : Prop)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule negation_intro(P : Prop)
  P := P ⊢ False
  ────────────────────────
  ⊢ ¬P
end;

rule negation_elim(P : Prop)
  ⊢ P;
  ⊢ ¬P
  ────────────────────────
  ⊢ False
end;

lemma dni(A : Prop, x := A)
  ⊢ ¬(¬A);
proof
  by wip(?goal);
wip;

lemma dni_imp(A : Prop)
  ⊢ A ⇒ ¬(¬A);
proof
  by wip(?goal);
wip;

lemma dni_combo(A : Prop, x := A)
  ⊢ ¬(¬A) ∧ (A ⇒ ¬(¬A));
proof
  by wip(?goal);
wip;
