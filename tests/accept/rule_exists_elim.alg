# Happy path for core rule `exists_elim`
# (|- exists x. P(x) ; (x : T, witness := P(x)) |- Q ==> |- Q).
# Q is chosen to be independently provable (a = a) so the witness hypothesis
# is not needed to discharge it.
import core(refl, exists_elim);

sort T : Sort;
op a : -> T;
axiom ex |- exists (x : T) st x = x;

lemma ex_elim
  |- a = a;
proof
  by exists_elim(T, _ = _, a = a) cases
    case
      |- exists (x : T) st x = x;
    proof
      by ex;
    qed;
    case
      x : T;
      witness := x = x;
      |- a = a;
    proof
      by refl(T, a);
    qed;
  qed;
qed;
