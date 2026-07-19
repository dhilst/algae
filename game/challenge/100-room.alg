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

rule or_elim(P Q R : Prop)
  ⊢ P ∨ Q;
  hp := P ⊢ R;
  hq := Q ⊢ R
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
  hp := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

lemma and_comm(A B : Prop, both := A ∧ B)
  ⊢ B ∧ A;
proof
  by wip(?goal);
wip;

lemma or_comm(A B : Prop, d := A ∨ B)
  ⊢ B ∨ A;
proof
  by wip(?goal);
wip;

lemma and_comm_imp(A B : Prop)
  ⊢ (A ∧ B) ⇒ (B ∧ A);
proof
  by wip(?goal);
wip;

lemma or_comm_imp(A B : Prop)
  ⊢ (A ∨ B) ⇒ (B ∨ A);
proof
  by wip(?goal);
wip;

lemma commute_all(A B : Prop)
  ⊢ ((A ∧ B) ⇒ (B ∧ A)) ∧ ((A ∨ B) ⇒ (B ∨ A));
proof
  by wip(?goal);
wip;
