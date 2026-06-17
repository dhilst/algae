# A parametric list module, parameterised by element type T.
sort List[T];

op nil : → List[T];
op cons : T × List[T] → List[T];
op head : List[T] → T;

axiom head_cons (x : T, xs : List[T]) head(cons(x, xs)) = x;
