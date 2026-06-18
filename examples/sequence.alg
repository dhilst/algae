# Finite sequences over Elem, built on the built-in Seq type and the ++
# concatenation operator. Concatenation is associative with the empty
# sequence as its unit.
sort Elem : Sort;

op empty : → Seq[Elem];
op single : Elem → Seq[Elem];

eq concat_assoc(xs ys zs : Seq[Elem]) (xs ++ ys) ++ zs = xs ++ (ys ++ zs);
eq concat_unit_left(xs : Seq[Elem]) empty() ++ xs = xs;
eq concat_unit_right(xs : Seq[Elem]) xs ++ empty() = xs;
