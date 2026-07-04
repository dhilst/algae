# Happy path for core rule `false_elim` (|- False ==> |- P).
import core(false_elim);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom absurd
  |- False;

lemma anything
  |- a = b;
proof
  by false_elim(a = b)
  then
    |- False;
    by absurd;
qed;
