# Happy path for core rule `forall_elim`
# (|- forall y. P(y) ==> |- P(x)).
# The witness x is an explicit third parameter, so it is supplied as the third
# argument; the conclusion P(x) = (x = x) then matches the goal.
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
