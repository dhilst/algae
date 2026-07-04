# Happy path for core rule `biconditional_elim_left` (|- P <=> Q ==> |- P => Q).
import core(biconditional_elim_left);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom equiv
  |- (a = a) <=> (b = b);

lemma fwd
  |- (a = a) => (b = b);
proof
  by biconditional_elim_left(a = a, b = b)
  then
    |- (a = a) <=> (b = b);
    by equiv;
qed;
