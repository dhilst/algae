# Something exists. Name it, then reach a truth that never needed the name.
import core(refl, exists_elim);

sort T : Sort;
op a : -> T;
axiom ex |- exists (x : T) st x = x;

lemma banish
  |- a = a;
proof
  by exists_elim(T, _ = _, a = a) cases
    case
      |- exists (x : T) st x = x;
      by ex;
    qed;
    case
      x : T;
      witness := x = x;
      |- a = a;
      by wip;
    wip;
  wip;
wip;
