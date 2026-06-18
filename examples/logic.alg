# Propositional natural deduction: introduction and elimination rules for
# conjunction (∧), disjunction (∨), implication (⟹), and negation (¬). Each
# rule premise is a named `case`; ⊥ (falsehood) is absurdity. Rules are parsed
# and type-checked structurally — no proof obligation is discharged.

# Conjunction --------------------------------------------------------------

rule and_intro(A : Prop, B : Prop)
  case left
    ⊢ A
  end;
  case right
    ⊢ B
  end;
  ─────────────────────────
  ⊢ A ∧ B
end;

rule and_elim_left(A : Prop, B : Prop)
  case conj
    ⊢ A ∧ B
  end;
  ─────────────────────────
  ⊢ A
end;

rule and_elim_right(A : Prop, B : Prop)
  case conj
    ⊢ A ∧ B
  end;
  ─────────────────────────
  ⊢ B
end;

# Disjunction --------------------------------------------------------------

rule or_intro_left(A : Prop, B : Prop)
  case left
    ⊢ A
  end;
  ─────────────────────────
  ⊢ A ∨ B
end;

rule or_intro_right(A : Prop, B : Prop)
  case right
    ⊢ B
  end;
  ─────────────────────────
  ⊢ A ∨ B
end;

# Case analysis: from A ∨ B, with C derivable from each side, conclude C.
rule or_elim(A : Prop, B : Prop, C : Prop)
  case disj
    ⊢ A ∨ B
  end;
  case on_left
    A ⊢ C
  end;
  case on_right
    B ⊢ C
  end;
  ─────────────────────────
  ⊢ C
end;

# Implication --------------------------------------------------------------

# Discharging a hypothesis: assume A, derive B, conclude A ⟹ B.
rule implies_intro(A : Prop, B : Prop)
  case discharge
    A ⊢ B
  end;
  ─────────────────────────
  ⊢ A ⟹ B
end;

# Modus ponens.
rule implies_elim(A : Prop, B : Prop)
  case imp
    ⊢ A ⟹ B
  end;
  case hyp
    ⊢ A
  end;
  ─────────────────────────
  ⊢ B
end;

# Negation: ¬A holds when assuming A leads to ⊥ ----------------------------

rule not_intro(A : Prop)
  case absurd
    A ⊢ ⊥
  end;
  ─────────────────────────
  ⊢ ¬ A
end;

rule not_elim(A : Prop)
  case hyp
    ⊢ A
  end;
  case neg
    ⊢ ¬ A
  end;
  ─────────────────────────
  ⊢ ⊥
end;
