# Happy path for core rule `backward` (|- P(a) ==> |- P(b)).
# With eq : a = b and motive P = (lambda k. k = k), P(a) = (a = a) and
# P(b) = (b = b). The goal is P(b) = (b = b); the premise is P(a) = (a = a).
import core(refl, backward);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma rw
  |- b = b;
proof
  by backward(T, a, b, ab, _ = _)
  then
    |- a = a;
    by refl(T, a);
qed;
