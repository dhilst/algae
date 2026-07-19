import core(forward, refl);

sort List : Sort → Sort;
op nil  : forall (A : Sort) st → List(A);
op cons : forall (A : Sort) st A * List(A) → List(A);
op app  : forall (A : Sort) st List(A) * List(A) → List(A);

axiom app_nil(A : Sort, ys : List(A))              ⊢ app(A, nil(A), ys) = ys;
axiom app_cons(A : Sort, x : A, xs ys : List(A))   ⊢ app(A, cons(A, x, xs), ys) = cons(A, x, app(A, xs, ys));

lemma app_left_nil(A : Sort, ys : List(A))
  ⊢ app(A, nil(A), ys) = ys;
proof
  by wip(?goal);
wip;

lemma app_singleton(A : Sort, x : A, ys : List(A))
  ⊢ app(A, cons(A, x, nil(A)), ys) = cons(A, x, ys);
proof
  by wip(?goal);
wip;
