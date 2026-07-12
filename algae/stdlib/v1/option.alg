import core(
  refl,
  backward,
  transitivity
);

import monad(
  Monad
);

sort Option : Sort → Sort;

op none : forall (A : Sort) st → Option(A);

op some : forall (A : Sort) st A → Option(A);

op return : forall (A : Sort) st A → Option(A);

op bind : forall (A B : Sort) st Option(A) * (A → Option(B)) → Option(B);

axiom return_def(
  A : Sort,
  x : A
)
  ⊢ return(A, x) = some(A, x);

axiom bind_none(
  A B : Sort,
  f : A → Option(B)
)
  ⊢ bind(A, B, none(A), f) = none(B);

axiom bind_some(
  A B : Sort,
  x : A,
  f : A → Option(B)
)
  ⊢ bind(A, B, some(A, x), f) = f(x);

rule option_cases(
  A : Sort,
  m : Option(A),
  P : Option(A) → Prop
)
  ⊢ P(none(A));
  x : A ⊢ P(some(A, x))
  ────────────────────────
  ⊢ P(m)
end;

model OptionMonad satisfies Monad(
  Option,
  return,
  bind
) iff laws
  law left_identity;
  proof
    by backward(
      Option(A),
      return(A, x),
      some(A, x),
      return_def(A, x),
      λ (o : Option(A)) st bind(A, B, o, f) = f(x)
    )
    then ⊢ bind(A, B, some(A, x), f) = f(x);
    by bind_some(A, B, x, f);
  qed;

  law right_identity;
  proof
    by option_cases(
      A,
      m,
      λ (o : Option(A)) st bind(A, A, o, λ (w : A) st return(A, w)) = o
    ) cases
      case
        ⊢ bind(A, A, none(A), λ (w : A) st return(A, w)) = none(A);
        by bind_none(A, A, λ (w : A) st return(A, w));
      qed;

      case
        x : A;
        ⊢ bind(A, A, some(A, x), λ (w : A) st return(A, w)) = some(A, x);
        by backward(
          Option(A),
          bind(A, A, some(A, x), λ (w : A) st return(A, w)),
          return(A, x),
          bind_some(A, A, x, λ (w : A) st return(A, w)),
          λ (o : Option(A)) st o = some(A, x)
        )
        then ⊢ return(A, x) = some(A, x);
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
        bind(B, C, bind(A, B, o, f), g)
        =
        bind(
          A, C, o,
          λ (x : A) st bind(B, C, f(x), g)
        )
    ) cases
      case
        ⊢ bind(B, C, bind(A, B, none(A), f), g)
           =
           bind(
             A, C, none(A),
             λ (x : A) st bind(B, C, f(x), g)
           );
        by backward(
          Option(B),
          bind(A, B, none(A), f),
          none(B),
          bind_none(A, B, f),
          λ (r : Option(B)) st
            bind(B, C, r, g)
            =
            bind(
              A, C, none(A),
              λ (x : A) st bind(B, C, f(x), g)
            )
        )
        then ⊢ bind(B, C, none(B), g) = bind(A, C, none(A), λ (x : A) st bind(B, C, f(x), g));
        by backward(
          Option(C),
          bind(A, C, none(A), λ (x : A) st bind(B, C, f(x), g)),
          none(C),
          bind_none(A, C, λ (x : A) st bind(B, C, f(x), g)),
          λ (r : Option(C)) st bind(B, C, none(B), g) = r
        )
        then ⊢ bind(B, C, none(B), g) = none(C);
        by bind_none(B, C, g);
      qed;

      case
        x : A;
        ⊢ bind(B, C, bind(A, B, some(A, x), f), g)
           =
           bind(
             A, C, some(A, x),
             λ (y : A) st bind(B, C, f(y), g)
           );
        by backward(
          Option(B),
          bind(A, B, some(A, x), f),
          f(x),
          bind_some(A, B, x, f),
          λ (r : Option(B)) st
            bind(B, C, r, g)
            =
            bind(
              A, C, some(A, x),
              λ (y : A) st bind(B, C, f(y), g)
            )
        )
        then ⊢ bind(B, C, f(x), g) = bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g));
        by backward(
          Option(C),
          bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g)),
          bind(B, C, f(x), g),
          bind_some(A, C, x, λ (y : A) st bind(B, C, f(y), g)),
          λ (r : Option(C)) st
            r = bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g))
        )
        then ⊢ bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g))
              = bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g));
        by refl(Option(C), bind(A, C, some(A, x), λ (y : A) st bind(B, C, f(y), g)));
      qed;
    qed;
  qed;
qed;
