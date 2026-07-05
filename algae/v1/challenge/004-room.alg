# a to b, b to c. Close the chain.
import core(transitivity);

sort T : Sort;
op a : -> T;
op b : -> T;
op c : -> T;
axiom ab |- a = b;
axiom bc |- b = c;

lemma banish
  |- a = c;
proof
  by transitivity(T, a, b, c) cases
    case
      |- a = b;
      by ab;
    qed;
    case
      |- b = c;
      by wip;
    wip;
  wip;
wip;
