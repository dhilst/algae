# Finite sequences over Elem, built on the built-in `Seq` type and the `++`
# concatenation operator. Concatenation is associative with the empty
# sequence as its unit; `length` is a homomorphism into ℕ.
sort Elem;

op empty : → Seq[Elem];
op single : Elem → Seq[Elem];
op length : Seq[Elem] → ℕ;

var xs, ys, zs : Seq[Elem];
var e : Elem;

axiom concat_assoc (xs ++ ys) ++ zs = xs ++ (ys ++ zs);
axiom concat_unit_left empty() ++ xs = xs;
axiom concat_unit_right xs ++ empty() = xs;
axiom length_empty length(empty()) = 0;
axiom length_single length(single(e)) = 1;
axiom length_concat length(xs ++ ys) = length(xs) + length(ys);
