# rewrite_r is sound only because the motive applied to the rewrite target must
# equal the goal: P(b) (and, for the reverse direction, P(a)) is checked against
# the current goal. Here the motive is `_ = z`, so P(b) = (b = z) and
# P(a) = (a = z), but the goal is `s(a) = s(a)`. Even though `e` validly proves
# `a = b`, neither P(b) nor P(a) is the goal, so the checker must reject this.
#
# (No equational axioms are declared, so definitional equality is plain
# beta/structural and cannot paper over the mismatch.)
import core(rewrite_r, reflexivity);

sort Nat : Sort;

op z : -> Nat;
op s : Nat -> Nat;

lemma bad(
  a b : Nat,
  e := a = b
)
  |- s(a) = s(a);
proof
  by rewrite_r(Nat, a, b, e, _ = z)
  case
    |- b = z;
  proof
    by reflexivity(Nat, b);
  qed;
qed;
