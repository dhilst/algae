# Sad: transitivity concludes an equation `x = z`. Applied here to a goal that is
# a conjunction (not an equation), the rule's conclusion cannot match the goal,
# so `transitivity` itself rejects the step.
#expect: tactic `transitivity`: rule conclusion does not match
import core(transitivity);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;
axiom bc |- b = c;

lemma bad
  |- a = c /\ a = c;
proof
  by transitivity(T, a, b, c) cases
    case
      |- a = b;
    proof
      by ab;
    qed;
    case
      |- b = c;
    proof
      by bc;
    qed;
  qed;
qed;
