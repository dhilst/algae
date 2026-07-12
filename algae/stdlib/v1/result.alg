import core(
  refl,
  backward
);

import monad(
  Monad
);

sort Result : Sort * Sort → Sort;

op ok : forall (A E : Sort) st A → Result(A, E);

op err : forall (A E : Sort) st E → Result(A, E);

op return : forall (A E : Sort) st A → Result(A, E);

op bind : forall (A B E : Sort) st Result(A, E) * (A → Result(B, E)) → Result(B, E);

axiom return_def(
  A E : Sort,
  x : A
)
  ⊢ return(A, E, x) = ok(A, E, x);

axiom bind_ok(
  A B E : Sort,
  x : A,
  f : A → Result(B, E)
)
  ⊢ bind(A, B, E, ok(A, E, x), f) = f(x);

axiom bind_err(
  A B E : Sort,
  e : E,
  f : A → Result(B, E)
)
  ⊢ bind(A, B, E, err(A, E, e), f) = err(B, E, e);

rule result_cases(
  A E : Sort,
  r : Result(A, E),
  P : Result(A, E) → Prop
)
  x : A ⊢ P(ok(A, E, x));
  e : E ⊢ P(err(A, E, e))
  ────────────────────────
  ⊢ P(r)
end;

# `Result(-, E)` is a monad for each fixed error type `E`. Because `Result` is
# binary while `Monad` expects a unary type constructor, the model supplies a
# type-abstracting `M = λ X. Result(X, E)` and error-fixed `return`/`bind` as
# dependent lambdas.
model ResultMonad satisfies Monad(
  λ (X : Sort) st Result(X, E),
  λ (X : Sort) st λ (v : X) st return(X, E, v),
  λ (X : Sort) st λ (Y : Sort) st
    λ (p : Result(X, E) * (X → Result(Y, E))) st bind(X, Y, E, p)
) iff laws
  law left_identity;
  proof
    by backward(
      Result(A, E),
      return(A, E, x),
      ok(A, E, x),
      return_def(A, E, x),
      λ (r : Result(A, E)) st bind(A, B, E, r, f) = f(x)
    )
    then ⊢ bind(A, B, E, ok(A, E, x), f) = f(x);
    by bind_ok(A, B, E, x, f);
  qed;

  law right_identity;
  proof
    by result_cases(
      A, E, m,
      λ (r : Result(A, E)) st bind(A, A, E, r, λ (w : A) st return(A, E, w)) = r
    ) cases
      case
        x : A;
        ⊢ bind(A, A, E, ok(A, E, x), λ (w : A) st return(A, E, w)) = ok(A, E, x);
        by backward(
          Result(A, E),
          bind(A, A, E, ok(A, E, x), λ (w : A) st return(A, E, w)),
          return(A, E, x),
          bind_ok(A, A, E, x, λ (w : A) st return(A, E, w)),
          λ (r : Result(A, E)) st r = ok(A, E, x)
        )
        then ⊢ return(A, E, x) = ok(A, E, x);
        by return_def(A, E, x);
      qed;

      case
        e : E;
        ⊢ bind(A, A, E, err(A, E, e), λ (w : A) st return(A, E, w)) = err(A, E, e);
        by bind_err(A, A, E, e, λ (w : A) st return(A, E, w));
      qed;
    qed;
  qed;

  law associativity;
  proof
    by result_cases(
      A, E, m,
      λ (r : Result(A, E)) st
        bind(B, C, E, bind(A, B, E, r, f), g)
        =
        bind(
          A, C, E, r,
          λ (x : A) st bind(B, C, E, f(x), g)
        )
    ) cases
      case
        x : A;
        ⊢ bind(B, C, E, bind(A, B, E, ok(A, E, x), f), g)
           =
           bind(
             A, C, E, ok(A, E, x),
             λ (y : A) st bind(B, C, E, f(y), g)
           );
        by backward(
          Result(B, E),
          bind(A, B, E, ok(A, E, x), f),
          f(x),
          bind_ok(A, B, E, x, f),
          λ (r : Result(B, E)) st
            bind(B, C, E, r, g)
            =
            bind(
              A, C, E, ok(A, E, x),
              λ (y : A) st bind(B, C, E, f(y), g)
            )
        )
        then ⊢ bind(B, C, E, f(x), g) = bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g));
        by backward(
          Result(C, E),
          bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g)),
          bind(B, C, E, f(x), g),
          bind_ok(A, C, E, x, λ (y : A) st bind(B, C, E, f(y), g)),
          λ (r : Result(C, E)) st
            r = bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g))
        )
        then ⊢ bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g))
              = bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g));
        by refl(Result(C, E), bind(A, C, E, ok(A, E, x), λ (y : A) st bind(B, C, E, f(y), g)));
      qed;

      case
        e : E;
        ⊢ bind(B, C, E, bind(A, B, E, err(A, E, e), f), g)
           =
           bind(
             A, C, E, err(A, E, e),
             λ (x : A) st bind(B, C, E, f(x), g)
           );
        by backward(
          Result(B, E),
          bind(A, B, E, err(A, E, e), f),
          err(B, E, e),
          bind_err(A, B, E, e, f),
          λ (r : Result(B, E)) st
            bind(B, C, E, r, g)
            =
            bind(
              A, C, E, err(A, E, e),
              λ (x : A) st bind(B, C, E, f(x), g)
            )
        )
        then ⊢ bind(B, C, E, err(B, E, e), g) = bind(A, C, E, err(A, E, e), λ (x : A) st bind(B, C, E, f(x), g));
        by backward(
          Result(C, E),
          bind(A, C, E, err(A, E, e), λ (x : A) st bind(B, C, E, f(x), g)),
          err(C, E, e),
          bind_err(A, C, E, e, λ (x : A) st bind(B, C, E, f(x), g)),
          λ (r : Result(C, E)) st bind(B, C, E, err(B, E, e), g) = r
        )
        then ⊢ bind(B, C, E, err(B, E, e), g) = err(C, E, e);
        by bind_err(B, C, E, e, g);
      qed;
    qed;
  qed;
qed;
