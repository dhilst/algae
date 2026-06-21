# Sad path: sum_reflect_right(P, Q) takes a premise Q and concludes `P \/ Q`.
# Here the single premise case asserts P (`a = a`) instead of Q (`b = b`), so
# the supplied subproof does not match the rule's premise and the checker must
# reject.
#expect: case goal does not match the generated subgoal
import adt(sum_reflect_right, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- a = a \/ b = b;
proof
  by sum_reflect_right(a = a, b = b)
  case
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
