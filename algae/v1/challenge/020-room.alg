# Two halves must both be spoken.
import core(and_intro, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma banish
  |- a = a /\ b = b;
proof
  by and_intro(a = a, b = b) cases
    case
      |- a = a;
      by wip;
    wip;
    case
      |- b = b;
      by wip;
    wip;
  wip;
wip;
