rule implication_intro(P Q : Prop)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule implication_elim(P Q : Prop)
  ⊢ P ⇒ Q;
  ⊢ P
  ────────────────────────
  ⊢ Q
end;

lemma mp(A B : Prop, f := A ⇒ B, x := A)
  ⊢ B;
proof
  by wip(?goal);
wip;

lemma imp_trans(A B C : Prop, f := A ⇒ B, g := B ⇒ C)
  ⊢ A ⇒ C;
proof
  by wip(?goal);
wip;

lemma imp_quad(A B C D : Prop, f := A ⇒ B, g := B ⇒ C, h := C ⇒ D)
  ⊢ A ⇒ D;
proof
  by wip(?goal);
wip;
