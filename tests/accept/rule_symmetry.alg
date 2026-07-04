# Happy path for core rule `symmetry` (|- x = y ==> |- y = x).
# Premise proves a = b; symmetry concludes b = a, which is the goal.
import core(symmetry);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma sym
  |- b = a;
proof
  by symmetry(T, a, b)
  then
    |- a = b;
    by ab;
qed;
