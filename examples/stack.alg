# A LIFO stack with error-reporting pop and top, in data-first notation.
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;  # popping or reading an empty stack

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Stack × Elem | Error;
op top : Stack → Elem | Error;

# x.f(a) is pipe-first application sugar for f(x, a), so
# s.push(e).pop reads as pop(push(s, e)).
eq push_pop(s : Stack, e : Elem) s.push(e).pop = (s, e);
eq push_top(s : Stack, e : Elem) s.push(e).top = e;
eq empty_pop empty().pop = empty_error;
eq empty_top empty().top = empty_error;
