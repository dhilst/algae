# Happy path for core rule `exists_intro` (|- P(x) ==> |- exists x. P(x)).
# Witness a; motive P = (lambda k. k = k); P(a) = (a = a) by reflexivity.
import core(reflexivity, exists_intro);

sort T : Sort;
op a : -> T;

lemma ex_intro
  |- exists (x : T) st x = x;
proof
  by exists_intro(T, _ = _, a)
  case
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
