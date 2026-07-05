# Happy path for core rule `negation_elim` (|- P ; |- ~P ==> |- False).
# Both premises are asserted by axioms (the theory is intentionally inconsistent).
import core(negation_elim);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom pos
  |- a = b;

axiom neg
  |- ~(a = b);

lemma contradiction
  |- False;
proof
  by negation_elim(a = b) cases
    case
      |- a = b;
      by pos;
    qed;
    case
      |- ~(a = b);
      by neg;
    qed;
  qed;
qed;
