# A parametric optional value (Maybe): either `none` or `some(x)`.
# `or_else` projects out the value, falling back to a default when empty.
param T : Sort;

sort Option : Sort → Sort;

op none : → Option[T];
op some : T → Option[T];
op or_else : Option[T] × T → T;

eq or_else_some(x d : T) or_else(some(x), d) = x;
eq or_else_none(d : T) or_else(none(), d) = d;
