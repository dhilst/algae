# A parametric pair (product) of an A and a B, with projections.
param A : Sort;
param B : Sort;

sort Pair : Sort → Sort → Sort;

op pair : A × B → Pair[A, B];
op fst : Pair[A, B] → A;
op snd : Pair[A, B] → B;

eq fst_pair(a : A, b : B) fst(pair(a, b)) = a;
eq snd_pair(a : A, b : B) snd(pair(a, b)) = b;
