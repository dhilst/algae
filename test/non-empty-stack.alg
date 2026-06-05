# A stack that always holds at least one element, in data-first notation.
# Both observers are total: top always has an element to return, and pop
# returns either the pair (rest, top) or, for a singleton, just the element.
sort NEStack, Elem;

op singleton : Elem → NEStack;
op push : NEStack × Elem → NEStack;
op pop : NEStack → NEStack × Elem | Elem;
op top : NEStack → Elem;
op size : NEStack → ℕ;

var s : NEStack;
var e : Elem;

# Popping undoes pushing; popping a singleton yields the lone element.
axiom s.push(e).pop = (s, e);
axiom singleton(e).pop = e;
# top reads the most recent element.
axiom s.push(e).top = e;
axiom singleton(e).top = e;
# size counts elements.
axiom s.push(e).size = s.size + 1;
axiom singleton(e).size = 1;
# The invariant that names the sort: a non-empty stack has at least one element.
axiom s.size ≥ 1;
