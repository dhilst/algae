# It offers you a pair and asks only for its right hand.
import core(and_right);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom conj |- a = a /\ b = b;

lemma banish
  |- b = b;
proof
  by and_right(a = a, b = b)
  then
    |- a = a /\ b = b;
    by wip;
wip;
