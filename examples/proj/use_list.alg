# Instantiate list at Elem, alias the namespace, and open two names.
include list with (T := Elem);
alias l = list;
open list (nil, cons);

sort Elem;
var e : Elem;
var xs : list::List[Elem];

axiom open_matches_qualified cons(e, xs) = list::cons(e, xs);
axiom alias_matches_qualified l::cons(e, xs) = list::cons(e, xs);
axiom head_def list::head(cons(e, xs)) = e;
