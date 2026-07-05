# Sad path: and_intro concludes `P /\ Q`, but the goal is the bare `a = a`, so
# the rule conclusion cannot match the goal and the checker must reject. The
# diagnostic shows both propositions (the rule's conclusion and the goal).
#expect: the rule concludes:  (a = a) ∧ (b = b)
import core(and_intro, refl);

sort T : Sort;
op a : -> T;
op b : -> T;

lemma bad
  |- a = a;
proof
  by and_intro(a = a, b = b) cases
    case
      |- a = a;
      by refl(T, a);
    qed;
    case
      |- b = b;
      by refl(T, b);
    qed;
  qed;
qed;
