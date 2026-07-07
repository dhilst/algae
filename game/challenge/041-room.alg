rule or_elim(P Q R : Prop)
  ⊢ P ∨ Q;
  P := P ⊢ R;
  Q := Q ⊢ R
  ────────────────────────
  ⊢ R
end;

rule contrapositive(P Q : Prop)
  ⊢ ¬Q ⇒ ¬P
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma idem(A : Prop, d := A ∨ A)
  ⊢ A;
proof
  by wip(?goal);
wip;

lemma contra_swap(A B : Prop, nn := ¬A ⇒ ¬B)
  ⊢ B ⇒ A;
proof
  by wip(?goal);
wip;
