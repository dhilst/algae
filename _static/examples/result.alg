import core(
  refl,
  rewrite_r
);

import monad(
  Monad
);

sort Result : Sort × Sort → Sort;

op ok : A → Result(A, E);

op err : E → Result(A, E);

op return : A → Result(A, E);

op bind : Result(A, E) × (A → Result(B, E)) → Result(B, E);

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
) iff props
  law left_identity;
  proof
    by rewrite_r(
      Result(A, E),
      return(x),
      ok(x),
      return_def(A, E, x),
      λ (r : Result(A, E)) st bind(r, f) = f(x)
    )
    case
      A B E : Sort;
      x : A;
      f : A → Result(B, E);
      ⊢ bind(ok(x), f) = f(x);
    proof
      by bind_ok(A, B, E, x, f);
    qed;
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
      proof
        by rewrite_r(
          Result(A, E),
          bind(ok(x), return),
          return(x),
          bind_ok(A, A, E, x, return),
          λ (r : Result(A, E)) st r = ok(x)
        )
        case
          A E : Sort;
          x : A;
          ⊢ return(x) = ok(x);
        proof
          by return_def(A, E, x);
        qed;
      qed;

      case
        A E : Sort;
        e : E;
        ⊢ bind(err(e), return) = err(e);
      proof
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
      proof
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
        case
          A B C E : Sort;
          x : A;
          f : A → Result(B, E);
          g : B → Result(C, E);
          ⊢ bind(f(x), g)
             =
             bind(
               ok(x),
               λ (y : A) st bind(f(y), g)
             );
        proof
          by bind_ok(
            A,
            C,
            E,
            x,
            λ (y : A) st bind(f(y), g)
          );
        qed;
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
      proof
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
        case
          A B C E : Sort;
          e : E;
          f : A → Result(B, E);
          g : B → Result(C, E);
          ⊢ bind(err(e), g)
             =
             bind(
               err(e),
               λ (x : A) st bind(f(x), g)
             );
        proof
          by bind_err(
            A,
            C,
            E,
            e,
            λ (x : A) st bind(f(x), g)
          );
        qed;
      qed;
    qed;
  qed;
qed;
