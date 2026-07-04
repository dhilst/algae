# Happy path for core rule `rewrite_l` (|- P(b) ==> |- P(a)).
# With eq : a = b and motive P = (lambda k. k = k), P(b) = (b = b) and
# P(a) = (a = a). The goal is P(a) = (a = a); the premise is P(b) = (b = b).
import core(refl, rewrite_l);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma rw
  |- a = a;
proof
  by rewrite_l(T, a, b, ab, _ = _)
  then
    |- b = b;
    by refl(T, b);
qed;
