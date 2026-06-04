sort Container, Elem;

op empty_container : → Container;
op insert          : Container × Elem → Container;
op contains        : Container × Elem → 𝔹;

var c : Container;
var e : Elem;

axiom contains(insert(c, e), e)      = true;
axiom contains(empty_container(), e) = false;
