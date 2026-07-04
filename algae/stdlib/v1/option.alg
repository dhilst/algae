import core(
  refl,
  rewrite_r,
  transitivity
);

import monad(
  Monad
);

sort None : Sort;

sort Option : Sort → Sort;

op none : → None;

op some : A → Option(A);

op return : A → Option(A);

op bind : Option(A) × (A → Option(B)) → Option(B);

axiom return_def(
  A : Sort,
  x : A
)
  ⊢ return(x) = some(x);

axiom bind_none(
  A B : Sort,
  f : A → Option(B)
)
  ⊢ bind(none, f) = none;

axiom bind_some(
  A B : Sort,
  x : A,
  f : A → Option(B)
)
  ⊢ bind(some(x), f) = f(x);

rule option_cases(
  A : Sort,
  m : Option(A),
  P : Option(A) → Prop
)
  ⊢ P(none);
  x : A ⊢ P(some(x))
  ────────────────────────
  ⊢ P(m)
end;

model OptionMonad satisfies Monad(
  A,
  B,
  C,
  Option,
  return,
  bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      Option(A),
      return(x),
      some(x),
      return_def(A, x),
      λ (o : Option(A)) st bind(o, f) = f(x)
    )
    then ⊢ bind(some(x), f) = f(x);
    by bind_some(A, B, x, f);
  qed;

  law right_identity;
  proof
    by option_cases(
      A,
      m,
      λ (o : Option(A)) st bind(o, return) = o
    ) cases
      case
        A : Sort;
        ⊢ bind(none, return) = none;
      proof
        by bind_none(A, A, return);
      qed;

      case
        A : Sort;
        x : A;
        ⊢ bind(some(x), return) = some(x);
      proof
        by rewrite_r(
          Option(A),
          bind(some(x), return),
          return(x),
          bind_some(A, A, x, return),
          λ (o : Option(A)) st o = some(x)
        )
        then ⊢ return(x) = some(x);
        by return_def(A, x);
      qed;
    qed;
  qed;

  law associativity;
  proof
    by option_cases(
      A,
      m,
      λ (o : Option(A)) st
        bind(bind(o, f), g)
        =
        bind(
          o,
          λ (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C : Sort;
        f : A → Option(B);
        g : B → Option(C);
        ⊢ bind(bind(none, f), g)
           =
           bind(
             none,
             λ (x : A) st bind(f(x), g)
           );
      proof
        by rewrite_r(
          Option(B),
          bind(none, f),
          none,
          bind_none(A, B, f),
          λ (r : Option(B)) st
            bind(r, g)
            =
            bind(
              none,
              λ (x : A) st bind(f(x), g)
            )
        )
        then ⊢ bind(none, g) = bind(none, λ (x : A) st bind(f(x), g));
        by rewrite_r(
          Option(C),
          bind(none, λ (x : A) st bind(f(x), g)),
          none,
          bind_none(A, C, λ (x : A) st bind(f(x), g)),
          λ (r : Option(C)) st bind(none, g) = r
        )
        then ⊢ bind(none, g) = none;
        by bind_none(B, C, g);
      qed;

      case
        A B C : Sort;
        x : A;
        f : A → Option(B);
        g : B → Option(C);
        ⊢ bind(bind(some(x), f), g)
           =
           bind(
             some(x),
             λ (y : A) st bind(f(y), g)
           );
      proof
        by rewrite_r(
          Option(B),
          bind(some(x), f),
          f(x),
          bind_some(A, B, x, f),
          λ (r : Option(B)) st
            bind(r, g)
            =
            bind(
              some(x),
              λ (y : A) st bind(f(y), g)
            )
        )
        then ⊢ bind(f(x), g) = bind(some(x), λ (y : A) st bind(f(y), g));
        by rewrite_r(
          Option(C),
          bind(some(x), λ (y : A) st bind(f(y), g)),
          bind(f(x), g),
          bind_some(A, C, x, λ (y : A) st bind(f(y), g)),
          λ (r : Option(C)) st
            r = bind(some(x), λ (y : A) st bind(f(y), g))
        )
        then ⊢ bind(some(x), λ (y : A) st bind(f(y), g))
              = bind(some(x), λ (y : A) st bind(f(y), g));
        by refl(Option(C), bind(some(x), λ (y : A) st bind(f(y), g)));
      qed;
    qed;
  qed;
qed;
