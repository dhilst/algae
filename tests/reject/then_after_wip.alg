# Sad path: `by wip` admits the goal and produces nothing; `then` cannot follow
# it (there is no remaining goal to continue).
#expect: syntax error
import core(refl);

sort T : Sort;
op a : -> T;

lemma bad
  |- a = a;
proof
  by wip
  then
    |- a = a;
  by refl(T, a);
qed;
