# The equation cuts the other way here.
import core(refl, rewrite_l);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma banish
  |- a = a;
proof
  by wip;
wip;
