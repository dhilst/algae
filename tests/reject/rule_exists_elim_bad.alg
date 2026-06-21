# Sad: exists_elim's conclusion is the bare meta-goal Q, so it matches any goal;
# the rule-specific check is on its generated subgoals. Its first subgoal must be
# the existential `exists (x : T) st P(x)`, but here the first case states `a = a`
# instead, so the case goal does not match the subgoal `exists_elim` generates.
#expect: case goal does not match the generated subgoal
import core(reflexivity, exists_elim);

sort T : Sort;
op a : -> T;
axiom ex |- exists (x : T) st x = x;

lemma bad
  |- a = a;
proof
  by exists_elim(T, _ = _, a = a) cases
    case
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
    case
      x : T;
      witness := x = x;
      |- a = a;
    proof
      by reflexivity(T, a);
    qed;
  qed;
qed;
