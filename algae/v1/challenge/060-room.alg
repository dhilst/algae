rule and_intro(P Q : Prop)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

rule and_left(P Q : Prop)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

rule and_right(P Q : Prop)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ Q
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

lemma and_comm(A B : Prop, both := A ∧ B)
  ⊢ B ∧ A;
proof
  by wip(?goal);
wip;

lemma and_comm_imp(A B : Prop)
  ⊢ (A ∧ B) ⇒ (B ∧ A);
proof
  by wip(?goal);
wip;

lemma and_comm_iff(A B : Prop)
  ⊢ (A ∧ B) ⇔ (B ∧ A);
proof
  by wip(?goal);
wip;
