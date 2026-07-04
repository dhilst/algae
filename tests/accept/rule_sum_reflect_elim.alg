# Happy path for adt rule `sum_reflect_elim`
# (|- P \/ Q ; (hyp P) |- R ; (hyp Q) |- R ==> |- R).
# The disjunction premise is asserted by axiom; both hypothesis cases share the
# goal `a = a`, provable independently by refl.
import adt(sum_reflect_elim, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom disj
  |- a = b \/ b = a;

lemma elim
  |- a = a;
proof
  by sum_reflect_elim(a = b, b = a, a = a) cases
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
