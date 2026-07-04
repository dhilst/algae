# Happy path for core rule `forall_elim`
# (|- forall y. P(y) ==> x : T |- P(x)).
# The conclusion carries an eigenvariable x in its context, so the lemma goal
# itself is stated under that context (lemma parameter x : T). The eigenvariable
# is supplied as the third argument so the conclusion P(x) = (x = x) matches the
# goal.
import core(forall_elim);

sort T : Sort;
axiom all_eq |- forall (y : T) st y = y;

lemma inst(x : T)
  |- x = x;
proof
  by forall_elim(T, _ = _, x)
  then
    |- forall (y : T) st y = y;
    by all_eq;
qed;
