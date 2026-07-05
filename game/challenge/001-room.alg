# "You know a = b," it hisses. "Then speak the mirror."
import core(symmetry);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma banish
  |- b = a;
proof
  by symmetry(T, a, b)
  then
    |- a = b;
    by wip;
wip;
