rule or_intro_right(P Q : Prop)
  ⊢ Q
  ────────────────────────
  ⊢ P ∨ Q
end;

lemma inject_right(A B : Prop, y := B)
  ⊢ A ∨ B;
proof
  by wip(?goal);
wip;
