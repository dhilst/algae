rule and_left(P Q : Prop)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

lemma take_left(A B : Prop, h := A ∧ B)
  ⊢ A;
proof
  by wip(?goal);
wip;
