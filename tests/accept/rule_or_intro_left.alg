# Happy path for core rule `or_intro_left` (|- P ==> |- P \/ Q).
import core(or_intro_left, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma disj
  |- a = a \/ b = b;
proof
  by or_intro_left(a = a, b = b)
  case
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
