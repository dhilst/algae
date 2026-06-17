# A parametric optional value (Maybe): either `none` or `some(x)`.
# `or_else` projects out the value, falling back to a default when empty.
sort Option[T];

op none : → Option[T];
op some : T → Option[T];
op or_else : Option[T] × T → T;

var x, d : T;

axiom or_else_some or_else(some(x), d) = x;
axiom or_else_none or_else(none(), d) = d;
