import core(forward, refl);

sort Stack : Sort → Sort;
op empty : forall (A : Sort) st → Stack(A);
op push  : forall (A : Sort) st A * Stack(A) → Stack(A);
op pop   : forall (A : Sort) st Stack(A) → Stack(A);
op top   : forall (A : Sort) st Stack(A) → A;

axiom top_push(A : Sort, x : A, s : Stack(A))  ⊢ top(A, push(A, x, s)) = x;
axiom pop_push(A : Sort, x : A, s : Stack(A))  ⊢ pop(A, push(A, x, s)) = s;

lemma top_one(A : Sort, a : A)
  ⊢ top(A, push(A, a, empty(A))) = a;
proof
  by wip(?goal);
wip;

lemma pop_then_top(A : Sort, a b : A)
  ⊢ top(A, pop(A, push(A, a, push(A, b, empty(A))))) = b;
proof
  by wip(?goal);
wip;
