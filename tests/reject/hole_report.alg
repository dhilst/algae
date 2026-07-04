# `by wip(?name)` reports a hole at the current goal (its type, context, and
# candidate tactics) and leaves the proof incomplete, so the module is rejected
# — but with a helpful hole diagnostic rather than a generic error.
#expect: found hole ?goal
import core(refl);

sort T : Sort;
op a : -> T;

lemma h
  |- a = a;
proof
  by wip(?goal);
wip;
