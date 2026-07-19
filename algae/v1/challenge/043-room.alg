rule negation_intro(P : Prop)
  hp := P ⊢ False
  ────────────────────────
  ⊢ ¬P
end;

rule implication_elim(P Q : Prop)
  ⊢ P ⇒ Q;
  ⊢ P
  ────────────────────────
  ⊢ Q
end;

rule negation_elim(P : Prop)
  ⊢ P;
  ⊢ ¬P
  ────────────────────────
  ⊢ False
end;

lemma neg_from_imp(A : Prop, f := A ⇒ False)
  ⊢ ¬A;
proof
  by wip(?goal);
wip;

lemma dni(A : Prop, x := A)
  ⊢ ¬(¬A);
proof
  by wip(?goal);
wip;
