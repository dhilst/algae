# Happy path for core rule `or_elim`
# (|- P \/ Q ; (hyp P) |- R ; (hyp Q) |- R ==> |- R).
# The disjunction premise is asserted by axiom; both hypothesis cases share the
# goal `a = a`, which is provable independently by refl.
import core(or_elim, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom disj
  |- a = b \/ b = a;

lemma elim
  |- a = a;
proof
  by or_elim(a = b, b = a, a = a) cases
    case
      |- a = b \/ b = a;
    proof
      by disj;
    qed;
    case
      hyp := a = b;
      |- a = a;
    proof
      by refl(T, a);
    qed;
    case
      hyp := b = a;
      |- a = a;
    proof
      by refl(T, a);
    qed;
  qed;
qed;
