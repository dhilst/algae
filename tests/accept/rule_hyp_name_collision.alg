# Regression: a rule whose premise names its hypothesis after a `Prop` parameter
# it references (`P := P`) must still yield a *usable* hypothesis — the internal
# name is freshened so a case-rename can't corrupt its proposition into `Pa := Pa`.
import core(implication_elim);

rule neg_intro(P : Prop)
  P := P ⊢ False
  ────────────────────────
  ⊢ ¬P
end;

lemma neg_from_imp(A : Prop, f := A ⇒ False)
  ⊢ ¬A;
proof
  by neg_intro(A)
  then
    f := A ⇒ False;
    Pa := A
    ⊢ False;
  by implication_elim(A, False) cases
    case
      f := A ⇒ False;
      Pa := A;
      ⊢ A ⇒ False;
      by f;
    qed;
    case
      f := A ⇒ False;
      Pa := A;
      ⊢ A;
      by Pa;
    qed;
  qed;
qed;
