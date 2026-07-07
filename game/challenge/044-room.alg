rule false_elim(P : Prop)
  ⊢ False
  ────────────────────────
  ⊢ P
end;

rule negation_elim(P : Prop)
  ⊢ P;
  ⊢ ¬P
  ────────────────────────
  ⊢ False
end;

rule contrapositive(P Q : Prop)
  ⊢ ¬Q ⇒ ¬P
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma explode(A C : Prop, x := A, nx := ¬A)
  ⊢ C;
proof
  by wip(?goal);
wip;

lemma contra_id(A : Prop)
  ⊢ A ⇒ A;
proof
  by wip(?goal);
wip;
