# No scaffold this time. Build the whole chain a = b = c yourself.
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
  by wip;
wip;
