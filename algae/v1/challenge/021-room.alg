# The right side will do.
import core(or_intro_right, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma banish
  |- a = a \/ b = b;
proof
  by or_intro_right(a = a, b = b)
  then
    |- b = b;
    by wip;
wip;
