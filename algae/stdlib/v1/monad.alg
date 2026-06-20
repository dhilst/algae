import core(
  reflexivity,
  rewrite_r,
  rewrite_l,
  transitivity
);

theory Functor(
  A B C : Sort,
  F : Sort → Sort,
  map : (A → B) × F(A) → F(B)
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
qed;

theory Applicative(
  A B C : Sort,
  F : Sort → Sort,
  pure : A → F(A),
  ap : F(A → B) × F(A) → F(B)
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
qed;

theory Monad(
  A B C : Sort,
  M : Sort → Sort,
  return : A → M(A),
  bind : M(A) × (A → M(B)) → M(B)
) laws
  law left_identity(
    x : A,
    f : A → M(B)
  )
    ⊢ bind(return(x), f) = f(x);

  law right_identity(
    m : M(A)
  )
    ⊢ bind(m, return) = m;

  law associativity(
    m : M(A),
    f : A → M(B),
    g : B → M(C)
  )
    ⊢ bind(bind(m, f), g)
       =
       bind(
         m,
         λ (x : A) st bind(f(x), g)
       );
qed;
