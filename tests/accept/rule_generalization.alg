# Happy path for core rule `generalization` (x : T |- P(x) ==> |- forall x. P(x)).
# The premise introduces eigenvariable x and proves x = x by reflexivity.
import core(reflexivity, generalization);

sort T : Sort;

lemma gen
  |- forall (x : T) st x = x;
proof
  by generalization(T, _ = _)
  case
    x : T;
    |- x = x;
  proof
    by reflexivity(T, x);
  qed;
qed;
