# A bare polymorphic operator (no type argument) is ill-typed as a term.
#expect: operator is missing type argument(s)
import option(none);

lemma e(A : Sort)
  |- none = none;
proof
  by wip;
wip;
