# Happy path for core rule `forall_intro` (x : T |- P(x) ==> |- forall x. P(x)).
# The premise introduces eigenvariable x and proves x = x by refl.
import core(refl, forall_intro);

sort T : Sort;

lemma gen
  |- forall (x : T) st x = x;
proof
  by forall_intro(T, _ = _)
  then
    x : T;
    |- x = x;
    by refl(T, x);
qed;
