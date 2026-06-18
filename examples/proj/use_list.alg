# Instantiate list at Elem, alias the namespace, and open two names.
include list with (T := Elem);
alias l = list;
open list (nil, cons);

sort Elem : Sort;

eq open_matches_qualified(e : Elem, xs : list::List[Elem])
  cons(e, xs) = list::cons(e, xs);
eq alias_matches_qualified(e : Elem, xs : list::List[Elem])
  l::cons(e, xs) = list::cons(e, xs);
eq head_def(e : Elem, xs : list::List[Elem])
  list::head(cons(e, xs)) = e;
