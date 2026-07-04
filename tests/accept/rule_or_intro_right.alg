# Happy path for core rule `or_intro_right` (|- Q ==> |- P \/ Q).
import core(or_intro_right, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma disj
  |- a = a \/ b = b;
proof
  by or_intro_right(a = a, b = b)
  then
    |- b = b;
    by refl(T, b);
qed;
