# The `wip` tactic ("work in progress") discharges a goal provisionally, to be
# finished later. It is viral: any subproof that uses it — directly or through a
# case that does — must be closed with `wip` instead of `qed`. The checker will
# use that marker later; for now it only enforces the qed/wip rule.
sort Nat : Sort;

op z : → Nat;
op s : Nat → Nat;
op add : Nat × Nat → Nat;

eq add_zero_left(n : Nat) add(z, n) = n;

rule induction(P : Nat → Prop)
  case base
    ⊢ P(z)
  end;
  case step
    n : Nat, P(n) ⊢ P(s(n))
  end;
  ─────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;

# A goal left for later: one `wip` step, closed with `wip`.
lemma todo(n : Nat)
  add(n, z) = n;
proof
  goal
    ⊢ add(n, z) = n
  by wip
  therefore
    ⊢ add(n, z) = n;
wip;

# Virality: each case is left `wip`, so each case is `wip`; that makes the
# `apply` `wip`, and the whole proof `wip` in turn.
lemma viral(n : Nat)
  add(n, z) = n;
proof
  goal
    ⊢ add(n, z) = n
  by apply induction(λ (n : Nat) => add(n, z) = n)
    case base
      goal
        ⊢ add(z, z) = z
      by wip
      therefore
        ⊢ add(z, z) = z;
    wip;
    case step
      goal
        n : Nat, ih := add(n, z) = n ⊢ add(s(n), z) = s(n)
      by wip
      therefore
        n : Nat, ih := add(n, z) = n ⊢ add(s(n), z) = s(n);
    wip;
  therefore done
  wip;
wip;
