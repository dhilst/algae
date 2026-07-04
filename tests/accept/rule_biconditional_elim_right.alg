# Happy path for core rule `biconditional_elim_right` (|- P <=> Q ==> |- Q => P).
import core(biconditional_elim_right);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom equiv
  |- (a = a) <=> (b = b);

lemma bwd
  |- (b = b) => (a = a);
proof
  by biconditional_elim_right(a = a, b = b)
  then
    |- (a = a) <=> (b = b);
    by equiv;
qed;
