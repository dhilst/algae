import core(
  refl,
  backward,
  transitivity
);

import monad(
  Monad
);

sort List : Sort → Sort;

op nil : forall (A : Sort) st → List(A);

op cons : forall (A : Sort) st A * List(A) → List(A);

op append : forall (A : Sort) st List(A) * List(A) → List(A);

op singleton : forall (A : Sort) st A → List(A);

op return : forall (A : Sort) st A → List(A);

op bind : forall (A B : Sort) st List(A) * (A → List(B)) → List(B);

axiom append_nil_left(
  A : Sort,
  xs : List(A)
)
  ⊢ append(A, nil(A), xs) = xs;

axiom append_cons_left(
  A : Sort,
  x : A,
  xs ys : List(A)
)
  ⊢ append(A, cons(A, x, xs), ys) = cons(A, x, append(A, xs, ys));

axiom singleton_def(
  A : Sort,
  x : A
)
  ⊢ singleton(A, x) = cons(A, x, nil(A));

axiom return_def(
  A : Sort,
  x : A
)
  ⊢ return(A, x) = singleton(A, x);

axiom bind_nil(
  A B : Sort,
  f : A → List(B)
)
  ⊢ bind(A, B, nil(A), f) = nil(B);

axiom bind_cons(
  A B : Sort,
  x : A,
  xs : List(A),
  f : A → List(B)
)
  ⊢ bind(A, B, cons(A, x, xs), f) = append(B, f(x), bind(A, B, xs, f));

axiom bind_singleton(
  A B : Sort,
  x : A,
  f : A → List(B)
)
  ⊢ bind(A, B, singleton(A, x), f) = f(x);

axiom append_nil_right(
  A : Sort,
  xs : List(A)
)
  ⊢ append(A, xs, nil(A)) = xs;

axiom append_associativity(
  A : Sort,
  xs ys zs : List(A)
)
  ⊢ append(A, append(A, xs, ys), zs) = append(A, xs, append(A, ys, zs));

axiom bind_append(
  A B : Sort,
  xs ys : List(A),
  f : A → List(B)
)
  ⊢ bind(A, B, append(A, xs, ys), f) = append(B, bind(A, B, xs, f), bind(A, B, ys, f));

rule list_induction(
  A : Sort,
  xs : List(A),
  P : List(A) → Prop
)
  ⊢ P(nil(A));
  x : A, rest : List(A), ih := P(rest) ⊢ P(cons(A, x, rest))
  ────────────────────────
  ⊢ P(xs)
end;

model ListMonad satisfies Monad(
  List,
  return,
  bind
) iff laws
  law left_identity;
  proof
    by backward(
      List(A),
      return(A, x),
      singleton(A, x),
      return_def(A, x),
      λ (xs : List(A)) st bind(A, B, xs, f) = f(x)
    )
    then ⊢ bind(A, B, singleton(A, x), f) = f(x);
    by bind_singleton(A, B, x, f);
  qed;

  law right_identity;
  proof
    by list_induction(
      A,
      m,
      λ (xs : List(A)) st bind(A, A, xs, λ (w : A) st return(A, w)) = xs
    ) cases
      case
        ⊢ bind(A, A, nil(A), λ (w : A) st return(A, w)) = nil(A);
        by bind_nil(A, A, λ (w : A) st return(A, w));
      qed;

      case
        x : A;
        rest : List(A);
        ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest;
        ⊢ bind(A, A, cons(A, x, rest), λ (w : A) st return(A, w)) = cons(A, x, rest);
        by backward(
          List(A),
          bind(A, A, cons(A, x, rest), λ (w : A) st return(A, w)),
          append(A, return(A, x), bind(A, A, rest, λ (w : A) st return(A, w))),
          bind_cons(A, A, x, rest, λ (w : A) st return(A, w)),
          λ (ys : List(A)) st ys = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ append(A, return(A, x), bind(A, A, rest, λ (w : A) st return(A, w))) = cons(A, x, rest);
        by backward(
          List(A),
          return(A, x),
          singleton(A, x),
          return_def(A, x),
          λ (ys : List(A)) st append(A, ys, bind(A, A, rest, λ (w : A) st return(A, w))) = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ append(A, singleton(A, x), bind(A, A, rest, λ (w : A) st return(A, w))) = cons(A, x, rest);
        by backward(
          List(A),
          bind(A, A, rest, λ (w : A) st return(A, w)),
          rest,
          ih,
          λ (ys : List(A)) st append(A, singleton(A, x), ys) = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ append(A, singleton(A, x), rest) = cons(A, x, rest);
        by backward(
          List(A),
          singleton(A, x),
          cons(A, x, nil(A)),
          singleton_def(A, x),
          λ (ys : List(A)) st append(A, ys, rest) = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ append(A, cons(A, x, nil(A)), rest) = cons(A, x, rest);
        by backward(
          List(A),
          append(A, cons(A, x, nil(A)), rest),
          cons(A, x, append(A, nil(A), rest)),
          append_cons_left(A, x, nil(A), rest),
          λ (ys : List(A)) st ys = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ cons(A, x, append(A, nil(A), rest)) = cons(A, x, rest);
        by backward(
          List(A),
          append(A, nil(A), rest),
          rest,
          append_nil_left(A, rest),
          λ (ys : List(A)) st cons(A, x, ys) = cons(A, x, rest)
        )
        then
          ih := bind(A, A, rest, λ (w : A) st return(A, w)) = rest
          ⊢ cons(A, x, rest) = cons(A, x, rest);
        by refl(List(A), cons(A, x, rest));
      qed;
    qed;
  qed;

  law associativity;
  proof
    by list_induction(
      A,
      m,
      λ (xs : List(A)) st
        bind(B, C, bind(A, B, xs, f), g)
        =
        bind(
          A, C, xs,
          λ (x : A) st bind(B, C, f(x), g)
        )
    ) cases
      case
        ⊢ bind(B, C, bind(A, B, nil(A), f), g)
           =
           bind(
             A, C, nil(A),
             λ (x : A) st bind(B, C, f(x), g)
           );
        by backward(
          List(B),
          bind(A, B, nil(A), f),
          nil(B),
          bind_nil(A, B, f),
          λ (ys : List(B)) st
            bind(B, C, ys, g)
            =
            bind(
              A, C, nil(A),
              λ (x : A) st bind(B, C, f(x), g)
            )
        )
        then ⊢ bind(B, C, nil(B), g) = bind(A, C, nil(A), λ (x : A) st bind(B, C, f(x), g));
        by backward(
          List(C),
          bind(A, C, nil(A), λ (x : A) st bind(B, C, f(x), g)),
          nil(C),
          bind_nil(A, C, λ (x : A) st bind(B, C, f(x), g)),
          λ (ys : List(C)) st bind(B, C, nil(B), g) = ys
        )
        then ⊢ bind(B, C, nil(B), g) = nil(C);
        by bind_nil(B, C, g);
      qed;

      case
        x : A;
        rest : List(A);
        ih := bind(B, C, bind(A, B, rest, f), g) = bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g));
        ⊢ bind(B, C, bind(A, B, cons(A, x, rest), f), g)
           =
           bind(
             A, C, cons(A, x, rest),
             λ (y : A) st bind(B, C, f(y), g)
           );
        by backward(
          List(B),
          bind(A, B, cons(A, x, rest), f),
          append(B, f(x), bind(A, B, rest, f)),
          bind_cons(A, B, x, rest, f),
          λ (ys : List(B)) st
            bind(B, C, ys, g)
            =
            bind(
              A, C, cons(A, x, rest),
              λ (y : A) st bind(B, C, f(y), g)
            )
        )
        then
          ih := bind(B, C, bind(A, B, rest, f), g) = bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g))
          ⊢ bind(B, C, append(B, f(x), bind(A, B, rest, f)), g)
              = bind(A, C, cons(A, x, rest), λ (y : A) st bind(B, C, f(y), g));
        by backward(
          List(C),
          bind(B, C, append(B, f(x), bind(A, B, rest, f)), g),
          append(C, bind(B, C, f(x), g), bind(B, C, bind(A, B, rest, f), g)),
          bind_append(B, C, f(x), bind(A, B, rest, f), g),
          λ (zs : List(C)) st
            zs
            =
            bind(
              A, C, cons(A, x, rest),
              λ (y : A) st bind(B, C, f(y), g)
            )
        )
        then
          ih := bind(B, C, bind(A, B, rest, f), g) = bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g))
          ⊢ append(C, bind(B, C, f(x), g), bind(B, C, bind(A, B, rest, f), g))
              = bind(A, C, cons(A, x, rest), λ (y : A) st bind(B, C, f(y), g));
        by backward(
          List(C),
          bind(B, C, bind(A, B, rest, f), g),
          bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g)),
          ih,
          λ (zs : List(C)) st
            append(C, bind(B, C, f(x), g), zs)
            =
            bind(
              A, C, cons(A, x, rest),
              λ (y : A) st bind(B, C, f(y), g)
            )
        )
        then
          ih := bind(B, C, bind(A, B, rest, f), g) = bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g))
          ⊢ append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g)))
              = bind(A, C, cons(A, x, rest), λ (y : A) st bind(B, C, f(y), g));
        by backward(
          List(C),
          bind(A, C, cons(A, x, rest), λ (y : A) st bind(B, C, f(y), g)),
          append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (y : A) st bind(B, C, f(y), g))),
          bind_cons(A, C, x, rest, λ (y : A) st bind(B, C, f(y), g)),
          λ (zs : List(C)) st
            append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (y : A) st bind(B, C, f(y), g))) = zs
        )
        then
          ih := bind(B, C, bind(A, B, rest, f), g) = bind(A, C, rest, λ (x : A) st bind(B, C, f(x), g))
          ⊢ append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (y : A) st bind(B, C, f(y), g)))
              = append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (y : A) st bind(B, C, f(y), g)));
        by refl(
          List(C),
          append(C, bind(B, C, f(x), g), bind(A, C, rest, λ (y : A) st bind(B, C, f(y), g)))
        );
      qed;
    qed;
  qed;
qed;
