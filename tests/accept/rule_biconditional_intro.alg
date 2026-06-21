# Happy path for core rule `biconditional_intro`
# (|- P => Q ; |- Q => P ==> |- P <=> Q).
# Both implication premises are asserted by axioms.
import core(biconditional_intro);

sort T : Sort;
op a : -> T;
op b : -> T;

axiom fwd
  |- (a = a) => (b = b);

axiom bwd
  |- (b = b) => (a = a);

lemma equiv
  |- (a = a) <=> (b = b);
proof
  by biconditional_intro(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
    proof
      by fwd;
    qed;
    case
      |- (b = b) => (a = a);
    proof
      by bwd;
    qed;
  qed;
qed;
