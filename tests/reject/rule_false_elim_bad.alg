# Sad path: false_elim's premise must be `False`, but the case discharges a
# different prop (a = a) and the goal-side premise subgoal is `a = a` not False,
# so the premise subgoal cannot match `False`.
#expect: case goal does not match the generated subgoal
import core(false_elim, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- a = b;
proof
  by false_elim(a = b)
  case
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
