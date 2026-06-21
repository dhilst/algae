# Sad path: implication_intro(P, Q) concludes `P => Q` ((b=b) => (a=a)), but the
# goal is the flipped `(a=a) => (b=b)`, so the rule conclusion cannot match.
#expect: rule conclusion does not match the current goal
import core(implication_intro, reflexivity);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- (a = a) => (b = b);
proof
  by implication_intro(b = b, a = a)
  case
    hyp := b = b;
    |- a = a;
  proof
    by reflexivity(T, a);
  qed;
qed;
