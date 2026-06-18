# A stack that always holds at least one element, in data-first notation.
# Both observers are total: top always has an element to return, and pop
# returns either the pair (rest, top) or, for a singleton, just the element.
sort NEStack : Sort;
sort Elem : Sort;

op singleton : Elem → NEStack;
op push : NEStack × Elem → NEStack;
op pop : NEStack → NEStack × Elem | Elem;
op top : NEStack → Elem;

# Popping undoes pushing; popping a singleton yields the lone element.
eq push_pop(s : NEStack, e : Elem) s.push(e).pop = (s, e);
eq singleton_pop(e : Elem) singleton(e).pop = e;
# top reads the most recent element.
eq push_top(s : NEStack, e : Elem) s.push(e).top = e;
eq singleton_top(e : Elem) singleton(e).top = e;
