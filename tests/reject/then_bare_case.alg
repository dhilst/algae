# Sad path: the old single-`case` continuation is no longer valid syntax —
# a `case` may appear only inside a `cases` block. Use `then` for one goal.
#expect: syntax error
import core(refl, rewrite_r);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom ab |- a = b;

lemma bad
  |- b = b;
proof
  by rewrite_r(T, a, b, ab, _ = _)
  case
    |- a = a;
    by refl(T, a);
  qed;
qed;
