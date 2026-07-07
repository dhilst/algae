rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma id(A : Prop)
  ⊢ A ⇒ A;
proof
  by wip(?goal);
wip;
