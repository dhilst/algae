# Named argument holes `?a`, `?b` are solved against the goal (here forced to
# `0 + n` and `n`); `then ?g` names the resulting subgoal. Incomplete.
#expect: Holes:
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  |- n = 0 + n;
proof
  by symmetry(Nat, ?a, ?b) then ?g;
wip;
