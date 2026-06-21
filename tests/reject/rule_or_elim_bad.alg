# Sad: or_elim's conclusion is the bare meta-goal R, so it matches any goal; the
# rule-specific check is on its generated subgoals. Its first subgoal must be the
# disjunction `P \/ Q`, but here the first case states `a = a` instead, so the
# case goal does not match the subgoal `or_elim` generates.
#expect: case goal does not match the generated subgoal
import core(or_elim, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom disj |- a = b \/ b = a;

lemma bad
  |- a = a;
proof
  by or_elim(a = b, b = a, a = a) cases
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
