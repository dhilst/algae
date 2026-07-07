sort Bool : Sort;

op true : → Bool;
op false : → Bool;

axiom refl(
  T : Sort,
  x : T
)
  ⊢ x = x;

# The two congruence rules rewrite one subterm of the goal using an equation.
# They are named for the direction the rewrite runs *at proof time* — remember
# that proofs go backward, from the goal toward its premises.
#
#   forward   takes a proof of `a = b` and replaces `a` with `b` in the goal
#             (the goal `P(a)` becomes the subgoal `P(b)` — following the
#             equation left-to-right, the way you'd read `a = b`).
#   backward  takes a proof of `a = b` and replaces `b` with `a` in the goal
#             (the goal `P(b)` becomes the subgoal `P(a)` — using the equation
#             right-to-left, against its natural reading).
#
# In both, `P : T → Prop` is the *motive*: a proposition with a hole marking
# exactly which subterm the equation lands on.

rule backward(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T → Prop
)
  ⊢ P(a)
  ────────────────────────
  ⊢ P(b)
end;

rule forward(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T → Prop
)
  ⊢ P(b)
  ────────────────────────
  ⊢ P(a)
end;

rule symmetry(
  T : Sort,
  x y : T
)
  ⊢ x = y
  ────────────────────────
  ⊢ y = x
end;

rule transitivity(
  T : Sort,
  x y z : T
)
  ⊢ x = y;
  ⊢ y = z
  ────────────────────────
  ⊢ x = z
end;

rule and_intro(
  P Q : Prop
)
  ⊢ P;
  ⊢ Q
  ────────────────────────
  ⊢ P ∧ Q
end;

rule and_left(
  P Q : Prop
)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ P
end;

rule and_right(
  P Q : Prop
)
  ⊢ P ∧ Q
  ────────────────────────
  ⊢ Q
end;

rule or_intro_left(
  P Q : Prop
)
  ⊢ P
  ────────────────────────
  ⊢ P ∨ Q
end;

rule or_intro_right(
  P Q : Prop
)
  ⊢ Q
  ────────────────────────
  ⊢ P ∨ Q
end;

rule or_elim(
  P Q R : Prop
)
  ⊢ P ∨ Q;
  P := P ⊢ R;
  Q := Q ⊢ R
  ────────────────────────
  ⊢ R
end;

rule implication_intro(
  P Q : Prop
)
  P := P ⊢ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule implication_elim(
  P Q : Prop
)
  ⊢ P ⇒ Q;
  ⊢ P
  ────────────────────────
  ⊢ Q
end;

rule negation_intro(
  P : Prop
)
  P := P ⊢ False
  ────────────────────────
  ⊢ ¬P
end;

rule negation_elim(
  P : Prop
)
  ⊢ P;
  ⊢ ¬P
  ────────────────────────
  ⊢ False
end;

rule false_elim(
  P : Prop
)
  ⊢ False
  ────────────────────────
  ⊢ P
end;

rule biconditional_intro(
  P Q : Prop
)
  ⊢ P ⇒ Q;
  ⊢ Q ⇒ P
  ────────────────────────
  ⊢ P ⇔ Q
end;

rule biconditional_elim_left(
  P Q : Prop
)
  ⊢ P ⇔ Q
  ────────────────────────
  ⊢ P ⇒ Q
end;

rule biconditional_elim_right(
  P Q : Prop
)
  ⊢ P ⇔ Q
  ────────────────────────
  ⊢ Q ⇒ P
end;

rule forall_elim(
  T : Sort,
  P : T → Prop
)
  ⊢ ∀ (y : T) st P(y)
  ────────────────────────
  x : T ⊢ P(x)
end;

rule forall_intro(
  T : Sort,
  P : T → Prop
)
  x : T ⊢ P(x)
  ────────────────────────
  ⊢ ∀ (x : T) st P(x)
end;

rule exists_intro(
  T : Sort,
  P : T → Prop,
  x : T
)
  ⊢ P(x)
  ────────────────────────
  ⊢ ∃ (x : T) st P(x)
end;

rule exists_elim(
  T : Sort,
  P : T → Prop,
  Q : Prop
)
  ⊢ ∃ (x : T) st P(x);
  x : T, witness := P(x) ⊢ Q
  ────────────────────────
  ⊢ Q
end;
