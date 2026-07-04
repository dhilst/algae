# Happy path for core rule `rewrite_r` (|- P(a) ==> |- P(b)).
# With eq : a = b and motive P = (lambda k. k = k), P(a) = (a = a) and
# P(b) = (b = b). The goal is P(b) = (b = b); the premise is P(a) = (a = a).
import core(refl, rewrite_r);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma rw
  |- b = b;
proof
  by rewrite_r(T, a, b, ab, _ = _)
  case
    |- a = a;
  proof
    by refl(T, a);
  qed;
qed;
