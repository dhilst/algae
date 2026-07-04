import core(
  refl,
  rewrite_r,
  transitivity
);

import monad(
  Monad
);

sort List : Sort → Sort;

op nil : → List(A);

op cons : A * List(A) → List(A);

op append : List(A) * List(A) → List(A);

op singleton : A → List(A);

op return : A → List(A);

op bind : List(A) * (A → List(B)) → List(B);

axiom append_nil_left(
  A : Sort,
  xs : List(A)
)
  ⊢ append(nil, xs) = xs;

axiom append_cons_left(
  A : Sort,
  x : A,
  xs ys : List(A)
)
  ⊢ append(cons(x, xs), ys) = cons(x, append(xs, ys));

axiom singleton_def(
  A : Sort,
  x : A
)
  ⊢ singleton(x) = cons(x, nil);

axiom return_def(
  A : Sort,
  x : A
)
  ⊢ return(x) = singleton(x);

axiom bind_nil(
  A B : Sort,
  f : A → List(B)
)
  ⊢ bind(nil, f) = nil;

axiom bind_cons(
  A B : Sort,
  x : A,
  xs : List(A),
  f : A → List(B)
)
  ⊢ bind(cons(x, xs), f) = append(f(x), bind(xs, f));

axiom bind_singleton(
  A B : Sort,
  x : A,
  f : A → List(B)
)
  ⊢ bind(singleton(x), f) = f(x);

axiom append_nil_right(
  A : Sort,
  xs : List(A)
)
  ⊢ append(xs, nil) = xs;

axiom append_associativity(
  A : Sort,
  xs ys zs : List(A)
)
  ⊢ append(append(xs, ys), zs) = append(xs, append(ys, zs));

axiom bind_append(
  A B : Sort,
  xs ys : List(A),
  f : A → List(B)
)
  ⊢ bind(append(xs, ys), f) = append(bind(xs, f), bind(ys, f));

rule list_induction(
  A : Sort,
  xs : List(A),
  P : List(A) → Prop
)
  ⊢ P(nil);
  x : A, rest : List(A), ih := P(rest) ⊢ P(cons(x, rest))
  ────────────────────────
  ⊢ P(xs)
end;

model ListMonad satisfies Monad(
  A,
  B,
  C,
  List,
  return,
  bind
) iff props
  law left_identity;
  proof
    by rewrite_r(
      List(A),
      return(x),
      singleton(x),
      return_def(A, x),
      λ (xs : List(A)) st bind(xs, f) = f(x)
    )
    then ⊢ bind(singleton(x), f) = f(x);
    by bind_singleton(A, B, x, f);
  qed;

  law right_identity;
  proof
    by list_induction(
      A,
      m,
      λ (xs : List(A)) st bind(xs, return) = xs
    ) cases
      case
        A : Sort;
        ⊢ bind(nil, return) = nil;
      proof
        by bind_nil(A, A, return);
      qed;

      case
        A : Sort;
        x : A;
        rest : List(A);
        ih := bind(rest, return) = rest;
        ⊢ bind(cons(x, rest), return) = cons(x, rest);
      proof
        by rewrite_r(
          List(A),
          bind(cons(x, rest), return),
          append(return(x), bind(rest, return)),
          bind_cons(A, A, x, rest, return),
          λ (ys : List(A)) st ys = cons(x, rest)
        )
        then ⊢ append(return(x), bind(rest, return)) = cons(x, rest);
        by rewrite_r(
          List(A),
          return(x),
          singleton(x),
          return_def(A, x),
          λ (ys : List(A)) st append(ys, bind(rest, return)) = cons(x, rest)
        )
        then ⊢ append(singleton(x), bind(rest, return)) = cons(x, rest);
        by rewrite_r(
          List(A),
          bind(rest, return),
          rest,
          ih,
          λ (ys : List(A)) st append(singleton(x), ys) = cons(x, rest)
        )
        then ⊢ append(singleton(x), rest) = cons(x, rest);
        by rewrite_r(
          List(A),
          singleton(x),
          cons(x, nil),
          singleton_def(A, x),
          λ (ys : List(A)) st append(ys, rest) = cons(x, rest)
        )
        then ⊢ append(cons(x, nil), rest) = cons(x, rest);
        by rewrite_r(
          List(A),
          append(cons(x, nil), rest),
          cons(x, append(nil, rest)),
          append_cons_left(A, x, nil, rest),
          λ (ys : List(A)) st ys = cons(x, rest)
        )
        then ⊢ cons(x, append(nil, rest)) = cons(x, rest);
        by rewrite_r(
          List(A),
          append(nil, rest),
          rest,
          append_nil_left(A, rest),
          λ (ys : List(A)) st cons(x, ys) = cons(x, rest)
        )
        then ⊢ cons(x, rest) = cons(x, rest);
        by refl(List(A), cons(x, rest));
      qed;
    qed;
  qed;

  law associativity;
  proof
    by list_induction(
      A,
      m,
      λ (xs : List(A)) st
        bind(bind(xs, f), g)
        =
        bind(
          xs,
          λ (x : A) st bind(f(x), g)
        )
    ) cases
      case
        A B C : Sort;
        f : A → List(B);
        g : B → List(C);
        ⊢ bind(bind(nil, f), g)
           =
           bind(
             nil,
             λ (x : A) st bind(f(x), g)
           );
      proof
        by rewrite_r(
          List(B),
          bind(nil, f),
          nil,
          bind_nil(A, B, f),
          λ (ys : List(B)) st
            bind(ys, g)
            =
            bind(
              nil,
              λ (x : A) st bind(f(x), g)
            )
        )
        then ⊢ bind(nil, g) = bind(nil, λ (x : A) st bind(f(x), g));
        by rewrite_r(
          List(C),
          bind(nil, λ (x : A) st bind(f(x), g)),
          nil,
          bind_nil(A, C, λ (x : A) st bind(f(x), g)),
          λ (ys : List(C)) st bind(nil, g) = ys
        )
        then ⊢ bind(nil, g) = nil;
        by bind_nil(B, C, g);
      qed;

      case
        A B C : Sort;
        x : A;
        rest : List(A);
        ih := bind(bind(rest, f), g) = bind(rest, λ (x : A) st bind(f(x), g));
        f : A → List(B);
        g : B → List(C);
        ⊢ bind(bind(cons(x, rest), f), g)
           =
           bind(
             cons(x, rest),
             λ (y : A) st bind(f(y), g)
           );
      proof
        by rewrite_r(
          List(B),
          bind(cons(x, rest), f),
          append(f(x), bind(rest, f)),
          bind_cons(A, B, x, rest, f),
          λ (ys : List(B)) st
            bind(ys, g)
            =
            bind(
              cons(x, rest),
              λ (y : A) st bind(f(y), g)
            )
        )
        then ⊢ bind(append(f(x), bind(rest, f)), g)
              = bind(cons(x, rest), λ (y : A) st bind(f(y), g));
        by rewrite_r(
          List(C),
          bind(append(f(x), bind(rest, f)), g),
          append(bind(f(x), g), bind(bind(rest, f), g)),
          bind_append(B, C, f(x), bind(rest, f), g),
          λ (zs : List(C)) st
            zs
            =
            bind(
              cons(x, rest),
              λ (y : A) st bind(f(y), g)
            )
        )
        then ⊢ append(bind(f(x), g), bind(bind(rest, f), g))
              = bind(cons(x, rest), λ (y : A) st bind(f(y), g));
        by rewrite_r(
          List(C),
          bind(bind(rest, f), g),
          bind(rest, λ (x : A) st bind(f(x), g)),
          ih,
          λ (zs : List(C)) st
            append(bind(f(x), g), zs)
            =
            bind(
              cons(x, rest),
              λ (y : A) st bind(f(y), g)
            )
        )
        then ⊢ append(bind(f(x), g), bind(rest, λ (x : A) st bind(f(x), g)))
              = bind(cons(x, rest), λ (y : A) st bind(f(y), g));
        by rewrite_r(
          List(C),
          bind(cons(x, rest), λ (y : A) st bind(f(y), g)),
          append(bind(f(x), g), bind(rest, λ (y : A) st bind(f(y), g))),
          bind_cons(A, C, x, rest, λ (y : A) st bind(f(y), g)),
          λ (zs : List(C)) st
            append(bind(f(x), g), bind(rest, λ (y : A) st bind(f(y), g))) = zs
        )
        then ⊢ append(bind(f(x), g), bind(rest, λ (y : A) st bind(f(y), g)))
              = append(bind(f(x), g), bind(rest, λ (y : A) st bind(f(y), g)));
        by refl(
          List(C),
          append(bind(f(x), g), bind(rest, λ (y : A) st bind(f(y), g)))
        );
      qed;
    qed;
  qed;
qed;
