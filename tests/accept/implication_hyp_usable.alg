# Regression: a hypothesis introduced by `implication_intro` must be usable as a
# rewrite equation (its eigenvar must not collide with the `P : Prop` param).
import core(implication_intro, backward, refl);

sort T : Sort;

lemma sym_imp(x y : T)
  |- x = y => y = x;
proof
  by implication_intro(x = y, y = x)
  then
    h := x = y;
    |- y = x;
    by backward(T, x, y, h, λ (t : T) st y = t)
    then |- y = y;
    by refl(T, y);
  qed;
