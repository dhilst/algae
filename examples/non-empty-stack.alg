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
axiom push_pop s.push(e).pop = (s, e);
axiom singleton_pop singleton(e).pop = e;
# top reads the most recent element.
axiom push_top s.push(e).top = e;
axiom singleton_top singleton(e).top = e;
# size counts elements.
axiom push_size s.push(e).size = s.size + 1;
axiom singleton_size singleton(e).size = 1;
# The invariant that names the sort: a non-empty stack has at least one element.
axiom non_empty s.size ≥ 1;
