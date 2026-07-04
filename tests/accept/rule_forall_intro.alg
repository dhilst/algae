# Happy path for core rule `forall_intro` (x : T |- P(x) ==> |- forall x. P(x)).
# The premise introduces eigenvariable x and proves x = x by refl.
import core(refl, forall_intro);

sort T : Sort;

lemma gen
  |- forall (x : T) st x = x;
proof
  by forall_intro(T, _ = _)
  case
    x : T;
    |- x = x;
  proof
    by refl(T, x);
  qed;
qed;
