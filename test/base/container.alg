sort Container, Elem;

op empty_container : → Container;
op insert : Container × Elem → Container;
op contains : Container × Elem → 𝔹;

var c : Container;
var e : Elem;

axiom contains_insert contains(insert(c, e), e) = true;
axiom contains_empty contains(empty_container(), e) = false;
