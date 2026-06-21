# Sad path: sum_reflect_elim's first premise must be the disjunction `P \/ Q`
# (`a = b \/ b = a`). Here the first case wrongly asserts the conclusion `a = a`
# instead of the disjunction, so the supplied subproof does not match the rule's
# premise and the checker must reject.
#expect: case goal does not match the generated subgoal
import adt(sum_reflect_elim, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom disj
  |- a = b \/ b = a;

lemma bad
  |- a = a;
proof
  by sum_reflect_elim(a = b, b = a, a = a) cases
    case
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
    case
      hyp := a = b;
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
    case
      hyp := b = a;
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
  qed;
qed;
