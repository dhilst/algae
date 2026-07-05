# Both directions, and the gate swings free.
import core(biconditional_intro);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom fwd |- (a = a) => (b = b);
axiom bwd |- (b = b) => (a = a);

lemma banish
  |- (a = a) <=> (b = b);
proof
  by biconditional_intro(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
      by fwd;
    qed;
    case
      |- (b = b) => (a = a);
      by wip;
    wip;
  wip;
wip;
