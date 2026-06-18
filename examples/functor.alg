# A functor F presented over a fixed object T via its endomaps. `map` lifts an
# endomap T → T to an endomap F[T] → F[T]. The two functor laws — preservation
# of identity and of composition — are proof obligations any instantiation must
# discharge. (The core is monomorphic, so the laws are stated at the single
# object T rather than across arbitrary A → B.)
param T : Sort;
param F : Sort → Sort;

op id   : T → T;
op comp : (T → T) × (T → T) → T → T;   # comp(g, f) is g after f
op map  : (T → T) × F[T] → F[T];

prop functor_identity(x : F[T])
  map(id, x) = x;

prop functor_composition(f g : T → T, x : F[T])
  map(comp(g, f), x) = map(g, map(f, x));
