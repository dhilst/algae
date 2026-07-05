# Whichever branch is true, the same truth waits at the end.
import core(or_elim, refl);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom disj |- a = b \/ b = a;

lemma banish
  |- a = a;
proof
  by or_elim(a = b, b = a, a = a) cases
    case
      |- a = b \/ b = a;
      by disj;
    qed;
    case
      hyp := a = b;
      |- a = a;
      by wip;
    wip;
    case
      hyp := b = a;
      |- a = a;
      by refl(T, a);
    qed;
  wip;
wip;
