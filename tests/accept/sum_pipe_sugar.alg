# `a | b` in a type position is shorthand for the adt `Sum(a, b)`, so the sum
# constructors `inl`/`inr` inhabit a `|`-typed operator's result.
import adt(inl, inr);

sort Error : Sort;
op err  : → Error;
op wrap : forall (A : Sort) st A → A | Error;

axiom wrap_def(A : Sort, x : A)  ⊢ wrap(A, x) = inl(A, Error, x);

lemma wraps_left(A : Sort, x : A)  ⊢ wrap(A, x) = inl(A, Error, x);
proof
  by wrap_def(A, x);
qed;
