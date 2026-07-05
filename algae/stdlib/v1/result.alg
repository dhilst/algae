import core(
  refl,
  rewrite_r
);

import monad(
  Monad
);

sort Result : Sort * Sort → Sort;

op ok : A → Result(A, E);

op err : E → Result(A, E);

op return : A → Result(A, E);

op bind : Result(A, E) * (A → Result(B, E)) → Result(B, E);

axiom return_def(
  A E : Sort,
  x : A
)
  ⊢ return(x) = ok(x);

axiom bind_ok(
  A B E : Sort,
  x : A,
  f : A → Result(B, E)
)
  ⊢ bind(ok(x), f) = f(x);

axiom bind_err(
  A B E : Sort,
  e : E,
  f : A → Result(B, E)
)
  ⊢ bind(err(e), f) = err(e);

rule result_cases(
  A E : Sort,
  r : Result(A, E),
  P : Result(A, E) → Prop
)
  x : A ⊢ P(ok(x));
  e : E ⊢ P(err(e))
  ────────────────────────
  ⊢ P(r)
end;

model ResultMonad satisfies Monad(
  A,
  B,
  C,
  λ (X : Sort) st Result(X, E),
  return,
  bind
) iff laws
  law left_identity;
  proof
    by rewrite_r(
      Result(A, E),
      return(x),
      ok(x),
      return_def(A, E, x),
      λ (r : Result(A, E)) st bind(r, f) = f(x)
    )
    then ⊢ bind(ok(x), f) = f(x);
    by bind_ok(A, B, E, x, f);
  qed;

  law right_identity;
  proof
    by result_cases(
      A,
      E,
      m,
      λ (r : Result(A, E)) st bind(r, return) = r
    ) cases
      case
        A E : Sort;
        x : A;
        ⊢ bind(ok(x), return) = ok(x);
        by rewrite_r(
          Result(A, E),
          bind(ok(x), return),
          return(x),
          bind_ok(A, A, E, x, return),
          λ (r : Result(A, E)) st r = ok(x)
        )
        then ⊢ return(x) = ok(x);
        by return_def(A, E, x);
      qed;

      case
        A E : Sort;
        e : E;
        ⊢ bind(err(e), return) = err(e);
        by bind_err(A, A, E, e, return);
      qed;
    qed;
  qed;

  law associativity;
  proof
    by result_cases(
      A,
      E,
      m,
      λ (r : Result(A, E)) st
        bind(bind(r, f), g)
        =
        bind(
          r,
          λ (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C E : Sort;
        x : A;
        f : A → Result(B, E);
        g : B → Result(C, E);
        ⊢ bind(bind(ok(x), f), g)
           =
           bind(
             ok(x),
             λ (y : A) st bind(f(y), g)
           );
        by rewrite_r(
          Result(B, E),
          bind(ok(x), f),
          f(x),
          bind_ok(A, B, E, x, f),
          λ (r : Result(B, E)) st
            bind(r, g)
            =
            bind(
              ok(x),
              λ (y : A) st bind(f(y), g)
            )
        )
        then ⊢ bind(f(x), g) = bind(ok(x), λ (y : A) st bind(f(y), g));
        by rewrite_r(
          Result(C, E),
          bind(ok(x), λ (y : A) st bind(f(y), g)),
          bind(f(x), g),
          bind_ok(A, C, E, x, λ (y : A) st bind(f(y), g)),
          λ (r : Result(C, E)) st
            r = bind(ok(x), λ (y : A) st bind(f(y), g))
        )
        then ⊢ bind(ok(x), λ (y : A) st bind(f(y), g))
              = bind(ok(x), λ (y : A) st bind(f(y), g));
        by refl(Result(C, E), bind(ok(x), λ (y : A) st bind(f(y), g)));
      qed;

      case
        A B C E : Sort;
        e : E;
        f : A → Result(B, E);
        g : B → Result(C, E);
        ⊢ bind(bind(err(e), f), g)
           =
           bind(
             err(e),
             λ (x : A) st bind(f(x), g)
           );
        by rewrite_r(
          Result(B, E),
          bind(err(e), f),
          err(e),
          bind_err(A, B, E, e, f),
          λ (r : Result(B, E)) st
            bind(r, g)
            =
            bind(
              err(e),
              λ (x : A) st bind(f(x), g)
            )
        )
        then ⊢ bind(err(e), g) = bind(err(e), λ (x : A) st bind(f(x), g));
        by rewrite_r(
          Result(C, E),
          bind(err(e), λ (x : A) st bind(f(x), g)),
          err(e),
          bind_err(A, C, E, e, λ (x : A) st bind(f(x), g)),
          λ (r : Result(C, E)) st bind(err(e), g) = r
        )
        then ⊢ bind(err(e), g) = err(e);
        by bind_err(B, C, E, e, g);
      qed;
    qed;
  qed;
qed;
