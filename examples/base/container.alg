# A minimal container with membership, included by other modules as a base.
sort Container : Sort;
sort Elem : Sort;

op empty_container : → Container;
op insert : Container × Elem → Container;
op contains : Container × Elem → 𝔹;

eq contains_insert(c : Container, e : Elem) contains(insert(c, e), e) = true;
eq contains_empty(e : Elem) contains(empty_container(), e) = false;
