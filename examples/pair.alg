# A parametric pair (product) of an A and a B, with projections.
sort Pair[A, B];

op pair : A × B → Pair[A, B];
op fst : Pair[A, B] → A;
op snd : Pair[A, B] → B;

var a : A;
var b : B;

axiom fst_pair fst(pair(a, b)) = a;
axiom snd_pair snd(pair(a, b)) = b;
