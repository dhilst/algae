# Happy path for core rule `implication_elim` (|- P => Q ; |- P ==> |- Q).
# Both premises are asserted by axioms; the conclusion `b = b` is the consequent.
import core(implication_elim);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom imp
  |- (a = a) => (b = b);

axiom ante
  |- a = a;

lemma modus_ponens
  |- b = b;
proof
  by implication_elim(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
    proof
      by imp;
    qed;
    case
      |- a = a;
    proof
      by ante;
    qed;
  qed;
qed;
