# Whichever branch is true, the same truth waits at the end.
import core(or_elim, refl);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom disj |- a = b \/ b = a;

lemma banish
  |- a = a;
proof
  by wip;
wip;
