# Happy path for adt rule `sum_reflect_right`
# (|- Q ==> |- P \/ Q). The single premise Q is proved by refl.
import adt(sum_reflect_right, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma inj
  |- a = a \/ b = b;
proof
  by sum_reflect_right(a = a, b = b)
  then
    |- b = b;
    by refl(T, b);
qed;
