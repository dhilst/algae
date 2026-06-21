# Sad: symmetry concludes an equation `y = x`. Applied here to a goal that is a
# conjunction (not an equation at all), the rule's conclusion cannot match the
# goal, so `symmetry` itself rejects the step.
#expect: tactic `symmetry`: rule conclusion does not match
import core(symmetry);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma bad
  |- a = b /\ b = a;
proof
  by symmetry(T, a, b)
  case
    |- a = b;
  proof
    by ab;
  qed;
qed;
