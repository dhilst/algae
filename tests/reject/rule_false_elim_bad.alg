# Sad path: false_elim's premise must be `False`, but the case discharges a
# different prop (a = a) and the goal-side premise subgoal is `a = a` not False,
# so the premise subgoal cannot match `False`.
#expect: `then` goal does not match the remaining subgoal
import core(false_elim, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- a = b;
proof
  by false_elim(a = b)
  then
    |- a = a;
    by refl(T, a);
qed;
