# Partial application of an operator is rejected (no currying); a function value
# must be written as a lambda.
#expect: currying is unsupported
import option(some);

lemma e(A : Sort)
  |- some(A) = some(A);
proof
  by wip;
wip;
