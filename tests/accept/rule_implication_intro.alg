# Happy path for core rule `implication_intro` ((hyp P) |- Q ==> |- P => Q).
# The goal `(b = b) => (a = a)` introduces hypothesis `b = b`, and the subgoal
# `a = a` is provable independently by reflexivity (the hypothesis is unused).
import core(implication_intro, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma imp
  |- (b = b) => (a = a);
proof
  by implication_intro(b = b, a = a)
  case
    hyp := b = b;
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
