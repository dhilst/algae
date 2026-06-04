# A LIFO stack with error-reporting pop and top.
sort Stack, Elem;
sort Error = {empty_error};  # popping or reading an empty stack

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Elem × Stack | Error;
op top : Stack → Elem | Error;

var s : Stack;
var e : Elem;

# LIFO behavior: pop returns the pair (top element, rest of the stack).
axiom pop(push(s, e)) = (e, s);
axiom top(push(s, e)) = e;
axiom pop(empty()) = empty_error;
axiom top(empty()) = empty_error;
