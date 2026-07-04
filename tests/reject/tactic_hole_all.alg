# `by ref?` holes every argument: the report lists each parameter (solved
# against the goal where possible) and the resulting subgoal. Incomplete.
#expect: found tactic hole
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  |- n = 0 + n;
proof
  by symmetry?;
wip;
