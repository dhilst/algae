rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma const_imp(A B : Prop, q := B)
  ⊢ A ⇒ B;
proof
  by wip(?goal);
wip;
