# Hole `_` sugar: an expression with `_` becomes a unary lambda whose binder
# type is inferred from the tactic parameter's function domain. All `_` in one
# expression share the single fresh binder.
import core(reflexivity, rewrite_r);

sort Nat : Sort;

op z : -> Nat;
op s : Nat -> Nat;

axiom s_collapses
  |- s(z) = z;

rule induction(
  P : Nat -> Prop
)
  |- P(z);
  n : Nat; ih := P(n); |- P(s(n))
  ------------------------
  |- forall (m : Nat) st P(m)
end;

# `_ = _` ==> lambda (k : Nat) st k = k   (both holes share one binder)
lemma all_self_eq
  |- forall (m : Nat) st m = m;
proof
  by induction(_ = _) cases
    case
      |- z = z;
    proof
      by reflexivity(Nat, z);
    qed;

    case
      n : Nat;
      ih := n = n;
      |- s(n) = s(n);
    proof
      by reflexivity(Nat, s(n));
    qed;
  qed;
qed;

# `s(_) = s(z)` ==> lambda (x : Nat) st s(x) = s(z)
lemma collapse
  |- s(s(z)) = s(z);
proof
  by rewrite_r(Nat, s(z), z, s_collapses, s(_) = s(z))
  case
    |- s(z) = s(z);
  proof
    by reflexivity(Nat, s(z));
  qed;
qed;
