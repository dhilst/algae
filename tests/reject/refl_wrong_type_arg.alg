# refl(T, x) requires x : T; passing b : B for parameter x : A must be rejected.
#expect: argument has type
import core(refl);

lemma foo(A B : Sort, b : B)
  |- b = b;
proof
  by refl(A, b);
qed;
