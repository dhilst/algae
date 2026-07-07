rule or_intro_left(P Q : Prop)
  ⊢ P
  ────────────────────────
  ⊢ P ∨ Q
end;

lemma inject_left(A B : Prop, x := A)
  ⊢ A ∨ B;
proof
  by wip(?goal);
wip;
