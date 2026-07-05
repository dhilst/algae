# Grant the hypothesis; the conclusion stands on its own.
import core(implication_intro, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma banish
  |- (b = b) => (a = a);
proof
  by implication_intro(b = b, a = a)
  then
    hyp := b = b;
    |- a = a;
    by wip;
wip;
