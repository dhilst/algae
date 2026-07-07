import core(
  refl,
  backward,
  forward,
  transitivity,
  and_intro,
  and_left,
  and_right,
  or_intro_left,
  or_intro_right,
  or_elim
);

sort Pair : Sort * Sort → Sort;

op pair : A * B → Pair(A, B);

op fst : Pair(A, B) → A;

op snd : Pair(A, B) → B;

axiom fst_pair(
  A B : Sort,
  x : A,
  y : B
)
  ⊢ fst(pair(x, y)) = x;

axiom snd_pair(
  A B : Sort,
  x : A,
  y : B
)
  ⊢ snd(pair(x, y)) = y;

rule pair_cases(
  A B : Sort,
  p : Pair(A, B),
  P : Pair(A, B) → Prop
)
  x : A, y : B ⊢ P(pair(x, y))
  ────────────────────────
  ⊢ P(p)
end;

rule product_reflect_intro(
  P Q : Prop
)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

rule product_reflect_left(
  P Q : Prop
)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

rule product_reflect_right(
  P Q : Prop
)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ Q
end;

sort Sum : Sort * Sort → Sort;

op inl : A → Sum(A, B);

op inr : B → Sum(A, B);

rule sum_cases(
  A B : Sort,
  s : Sum(A, B),
  P : Sum(A, B) → Prop
)
  x : A ⊢ P(inl(x));
  y : B ⊢ P(inr(y))
  ────────────────────────
  ⊢ P(s)
end;

rule sum_reflect_left(
  P Q : Prop
)
  ⊢ P
  ────────────────────────
  ⊢ P ∨ Q
end;

rule sum_reflect_right(
  P Q : Prop
)
  ⊢ Q
  ────────────────────────
  ⊢ P ∨ Q
end;

rule sum_reflect_elim(
  P Q R : Prop
)
  ⊢ P ∨ Q;
  P := P ⊢ R;
  Q := Q ⊢ R
  ────────────────────────
  ⊢ R
end;
