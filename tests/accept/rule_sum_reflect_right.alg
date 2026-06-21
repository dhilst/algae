# Happy path for adt rule `sum_reflect_right`
# (|- Q ==> |- P \/ Q). The single premise Q is proved by reflexivity.
import adt(sum_reflect_right, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma inj
  |- a = a \/ b = b;
proof
  by sum_reflect_right(a = a, b = b)
  case
    |- b = b;
  proof
    by reflexivity(T, b);
  qed;
qed;
