# `|` requires adt (for `Sum`); without it, the sum type is unresolved.
#expect: require importing `adt`
import core(refl);
sort A : Sort;
sort Error : Sort;
op f : -> A | Error;
lemma l |- f = f;
proof by wip; wip;
