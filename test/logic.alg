# Propositional natural deduction.
#
# Introduction and elimination rules for conjunction (∧), disjunction (∨),
# implication (⟹), and negation (¬), with a few derived lemmas. Propositions
# are the built-in Prop type; ⊥ (falsehood) is absurdity. Connectives combine
# Prop operands. As elsewhere, rules and proofs are parsed and type-checked
# structurally — no proof obligation is discharged.

var A, B, C : Prop;

# Conjunction --------------------------------------------------------------

rule and_intro(A : Prop, B : Prop)
  ⊢ A
  ⊢ B
  ─────
  ⊢ A ∧ B
end

rule and_elim_left(A : Prop, B : Prop)
  ⊢ A ∧ B
  ─────
  ⊢ A
end

rule and_elim_right(A : Prop, B : Prop)
  ⊢ A ∧ B
  ─────
  ⊢ B
end

# Disjunction --------------------------------------------------------------

rule or_intro_left(A : Prop, B : Prop)
  ⊢ A
  ─────
  ⊢ A ∨ B
end

rule or_intro_right(A : Prop, B : Prop)
  ⊢ B
  ─────
  ⊢ A ∨ B
end

# Case analysis: from A ∨ B, with C derivable from each side, conclude C.
# A proof's `case` names each premise's hypothesis (here `ha` / `hb`).
rule or_elim(A : Prop, B : Prop, C : Prop)
  ⊢ A ∨ B
  A ⊢ C
  B ⊢ C
  ─────
  ⊢ C
end

# Implication --------------------------------------------------------------

# Discharging a hypothesis: assume A, derive B, conclude A ⟹ B.
rule implies_intro(A : Prop, B : Prop)
  A ⊢ B
  ─────
  ⊢ A ⟹ B
end

# Modus ponens.
rule implies_elim(A : Prop, B : Prop)
  ⊢ A ⟹ B
  ⊢ A
  ─────
  ⊢ B
end

# Negation: ¬A holds when assuming A leads to ⊥ -----------------------------

rule not_intro(A : Prop)
  A ⊢ ⊥
  ─────
  ⊢ ¬ A
end

rule not_elim(A : Prop)
  ⊢ A
  ⊢ ¬ A
  ─────
  ⊢ ⊥
end

# Derived lemmas -----------------------------------------------------------

# Conjunction commutes: from A ∧ B derive B ∧ A. Build B ∧ A with and_intro,
# discharging each conjunct by the matching elimination on A ∧ B. Each case
# states the subgoal it proves.
lemma and_comm A ∧ B ⊢ B ∧ A;
proof
  apply and_intro(B, A);

  case [⊢ B]
    apply and_elim_right(A, B);
    case [⊢ A ∧ B]
      A ∧ B;
    qed;
  qed;

  case [⊢ A]
    apply and_elim_left(A, B);
    case [⊢ A ∧ B]
      A ∧ B;
    qed;
  qed;
qed;

# Implication is reflexive: ⊢ A ⟹ A. The case names the discharged hypothesis
# `h` and states the sequent `h := A ⊢ A`.
lemma implies_refl ⊢ A ⟹ A;
proof
  apply implies_intro(A, A);
  case [h := A ⊢ A]
    A;
  qed;
qed;

# Modus ponens as a lemma: from A and A ⟹ B derive B.
lemma modus_ponens A, A ⟹ B ⊢ B;
proof
  apply implies_elim(A, B);
  case [⊢ A ⟹ B]
    A ⟹ B;
  qed;
  case [⊢ A]
    A;
  qed;
qed;

# A and ¬A are contradictory.
lemma contradiction A, ¬ A ⊢ ⊥;
proof
  apply not_elim(A);
  case [⊢ A]
    A;
  qed;
  case [⊢ ¬ A]
    ¬ A;
  qed;
qed;
