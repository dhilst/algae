rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule and_left(P Q : Prop)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

rule contrapositive(P Q : Prop)
  ⊢ ¬Q ⇒ ¬P
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma proj(A B : Prop)
  ⊢ (A ∧ B) ⇒ A;
proof
  by wip(?goal);
wip;

lemma contra_direct(A B : Prop, nn := ¬B ⇒ ¬A)
  ⊢ A ⇒ B;
proof
  by wip(?goal);
wip;
