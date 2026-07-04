# Happy path for adt rule `sum_reflect_left`
# (|- P ==> |- P \/ Q). The single premise P is proved by refl.
import adt(sum_reflect_left, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma inj
  |- a = a \/ b = b;
proof
  by sum_reflect_left(a = a, b = b)
  case
    |- a = a;
  proof
    by refl(T, a);
  qed;
qed;
