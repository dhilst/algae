# Sad path: `cases` is for branching (two or more goals). A tactic producing a
# single goal must continue with `then`, not a one-branch `cases`.
#expect: `cases` is for branching
import core(refl, rewrite_r);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma bad
  |- b = b;
proof
  by rewrite_r(T, a, b, ab, _ = _) cases
    case
      |- a = a;
      by refl(T, a);
    qed;
  qed;
qed;
