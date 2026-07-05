# Rewrite the world along an equation until the goal appears.
import core(refl, rewrite_r);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma banish
  |- b = b;
proof
  by wip;
wip;
