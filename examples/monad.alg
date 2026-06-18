# A monad M over a fixed object T, in Kleisli (return/bind) form. `ret` injects a
# value; `bind` sequences a computation with a continuation T → M[T]. The three
# monad laws are proof obligations any instantiation must discharge. (The core is
# monomorphic, so the laws are stated at the single object T.)
param T : Sort;
param M : Sort → Sort;

op ret  : T → M[T];
op bind : M[T] × (T → M[T]) → M[T];

prop left_identity(x : T, f : T → M[T])
  bind(ret(x), f) = f(x);

prop right_identity(m : M[T])
  bind(m, ret) = m;

prop associativity(m : M[T], f g : T → M[T])
  bind(bind(m, f), g) = bind(m, λ (x : T) => bind(f(x), g));
