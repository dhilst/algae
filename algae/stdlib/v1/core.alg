sort Bool : Sort;

op true : -> Bool;
op false : -> Bool;

axiom reflexivity(
  T : Sort,
  x : T
)
  |- x = x;

rule rewrite_r(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T -> Prop
)
  |- P(a)

  ------------------------

  |- P(b)
end;

rule rewrite_l(
  T : Sort,
  a b : T,
  eq := a = b,
  P : T -> Prop
)
  |- P(b)

  ------------------------

  |- P(a)
end;

rule symmetry(
  T : Sort,
  x y : T
)
  |- x = y

  ------------------------

  |- y = x
end;

rule transitivity(
  T : Sort,
  x y z : T
)
  |- x = y;

  |- y = z

  ------------------------

  |- x = z
end;

rule and_intro(
  P Q : Prop
)
  |- P;

  |- Q

  ------------------------

  |- P /\ Q
end;

rule and_left(
  P Q : Prop
)
  |- P /\ Q

  ------------------------

  |- P
end;

rule and_right(
  P Q : Prop
)
  |- P /\ Q

  ------------------------

  |- Q
end;

rule or_intro_left(
  P Q : Prop
)
  |- P

  ------------------------

  |- P \/ Q
end;

rule or_intro_right(
  P Q : Prop
)
  |- Q

  ------------------------

  |- P \/ Q
end;

rule or_elim(
  P Q R : Prop
)
  |- P \/ Q;

  P := P |- R;

  Q := Q |- R

  ------------------------

  |- R
end;

rule implication_intro(
  P Q : Prop
)
  P := P |- Q

  ------------------------

  |- P => Q
end;

rule implication_elim(
  P Q : Prop
)
  |- P => Q;

  |- P

  ------------------------

  |- Q
end;

rule negation_intro(
  P : Prop
)
  P := P |- False

  ------------------------

  |- ~P
end;

rule negation_elim(
  P : Prop
)
  |- P;

  |- ~P

  ------------------------

  |- False
end;

rule false_elim(
  P : Prop
)
  |- False

  ------------------------

  |- P
end;

rule biconditional_intro(
  P Q : Prop
)
  |- P => Q;

  |- Q => P

  ------------------------

  |- P <=> Q
end;

rule biconditional_elim_left(
  P Q : Prop
)
  |- P <=> Q

  ------------------------

  |- P => Q
end;

rule biconditional_elim_right(
  P Q : Prop
)
  |- P <=> Q

  ------------------------

  |- Q => P
end;

rule instantiation(
  T : Sort,
  P : T -> Prop,
  x : T
)
  |- forall (y : T) st P(y)

  ------------------------

  |- P(x)
end;

rule generalization(
  T : Sort,
  x : T,
  P : T -> Prop
)
  |- P(x)

  ------------------------

  |- forall (x : T) st P(x)
end;

rule exists_intro(
  T : Sort,
  P : T -> Prop,
  x : T
)
  |- P(x)

  ------------------------

  |- exists (x : T) st P(x)
end;

rule exists_elim(
  T : Sort,
  P : T -> Prop,
  Q : Prop
)
  |- exists (x : T) st P(x);

  x : T, witness := P(x) |- Q

  ------------------------

  |- Q
end;
