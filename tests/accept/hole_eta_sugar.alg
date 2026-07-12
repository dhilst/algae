# `_` sugar: `some(A, _)` eta-expands to `λ (v : A) st some(A, v)`, so the two
# forms are definitionally equal.
import option(some, bind, refl);

lemma e(A : Sort, m : Option(A))
  |- bind(A, A, m, some(A, _)) = bind(A, A, m, λ (v : A) st some(A, v));
proof
  by refl(Option(A), bind(A, A, m, λ (v : A) st some(A, v)));
qed;
