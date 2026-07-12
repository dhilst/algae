# some(B, x) applies the constructor at sort B but the value x : A, so the
# goal term is ill-typed.
#expect: ill-typed goal
import option(some);

lemma e(A B : Sort, x : A)
  |- some(B, x) = some(B, x);
proof
  by wip;
wip;
