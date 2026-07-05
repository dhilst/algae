# One truth is enough to tip the balance.
import core(or_intro_left, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma banish
  |- a = a \/ b = b;
proof
  by or_intro_left(a = a, b = b)
  then
    |- a = a;
    by wip;
wip;
