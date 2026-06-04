# A LIFO stack with error-reporting pop and top.
sort Stack, Elem;
sort Error = {empty_error};  # popping or reading an empty stack

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Stack | Error;
op top : Stack → Elem | Error;

var s : Stack;
var e : Elem;

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
