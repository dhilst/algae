# A LIFO stack with error-reporting pop and top, in data-first notation.
sort Stack, Elem;
sort Error = {empty_error};  # popping or reading an empty stack

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Stack × Elem | Error;
op top : Stack → Elem | Error;

var s : Stack;
var e : Elem;

# x.f(a) is pipe-first application sugar for f(x, a), so
# s.push(e).pop reads as pop(push(s, e)).
axiom push_pop s.push(e).pop = (s, e);
axiom push_top s.push(e).top = e;
axiom empty_pop empty().pop = empty_error;
axiom empty_top empty().top = empty_error;
