import core(
  refl,
  backward,
  forward,
  transitivity
);

theory Functor(
  A B C : Sort,
  F : Sort → Sort,
  map : (A → B) * F(A) → F(B)
) laws
  law identity(
    x : F(A)
  )
    ⊢ map(λ (a : A) st a, x) = x;

  law composition(
    f : A → B,
    g : B → C,
    x : F(A)
  )
    ⊢ map(
         λ (a : A) st g(f(a)),
         x
       )
       =
       map(
         g,
         map(f, x)
       );
end;

theory Applicative(
  A B C : Sort,
  F : Sort → Sort,
  pure : A → F(A),
  ap : F(A → B) * F(A) → F(B)
) laws
  law identity(
    v : F(A)
  )
    ⊢ ap(pure(λ (x : A) st x), v) = v;

  law homomorphism(
    f : A → B,
    x : A
  )
    ⊢ ap(pure(f), pure(x)) = pure(f(x));

  law interchange(
    u : F(A → B),
    y : A
  )
    ⊢ ap(u, pure(y))
       =
       ap(
         pure(λ (f : A → B) st f(y)),
         u
       );

  law composition(
    u : F(B → C),
    v : F(A → B),
    w : F(A)
  )
    ⊢ ap(
         ap(
           ap(
             pure(λ (f : B → C) st λ (g : A → B) st λ (x : A) st f(g(x))),
             u
           ),
           v
         ),
         w
       )
       =
       ap(u, ap(v, w));
end;

theory Monad(
  M : Sort → Sort,
  return : forall (X : Sort) st X → M(X),
  bind : forall (X Y : Sort) st M(X) * (X → M(Y)) → M(Y)
) laws
  law left_identity(
    A B : Sort,
    x : A,
    f : A → M(B)
  )
    ⊢ bind(A, B, return(A, x), f) = f(x);

  law right_identity(
    A : Sort,
    m : M(A)
  )
    ⊢ bind(A, A, m, λ (x : A) st return(A, x)) = m;

  law associativity(
    A B C : Sort,
    m : M(A),
    f : A → M(B),
    g : B → M(C)
  )
    ⊢ bind(B, C, bind(A, B, m, f), g)
       =
       bind(
         A, C, m,
         λ (x : A) st bind(B, C, f(x), g)
       );
end;
