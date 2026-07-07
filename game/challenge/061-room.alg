rule or_elim(P Q R : Prop)
  ⊢ P ∨ Q;
  P := P ⊢ R;
  Q := Q ⊢ R
  ────────────────────────
  ⊢ R
end;

rule or_intro_left(P Q : Prop)
  ⊢ P
  ────────────────────────
  ⊢ P ∨ Q
end;

rule or_intro_right(P Q : Prop)
  ⊢ Q
  ────────────────────────
  ⊢ P ∨ Q
end;

rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule biconditional_intro(P Q : Prop)
  ⊢ P ⇒ Q;
  ⊢ Q ⇒ P
  ────────────────────────
  ⊢ P ⇔ Q
end;

lemma or_comm(A B : Prop, d := A ∨ B)
  ⊢ B ∨ A;
proof
  by wip(?goal);
wip;

lemma or_comm_imp(A B : Prop)
  ⊢ (A ∨ B) ⇒ (B ∨ A);
proof
  by wip(?goal);
wip;

lemma or_comm_iff(A B : Prop)
  ⊢ (A ∨ B) ⇔ (B ∨ A);
proof
  by wip(?goal);
wip;
