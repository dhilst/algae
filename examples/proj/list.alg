# A parametric list module, parameterised by element type T.
param T : Sort;

sort List : Sort → Sort;

op nil : → List[T];
op cons : T × List[T] → List[T];
op head : List[T] → T;

eq head_cons(x : T, xs : List[T]) head(cons(x, xs)) = x;
