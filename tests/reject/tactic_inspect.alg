# A trailing `?` on a fully concrete application applies the tactic and suggests
# the next step (a paste-able `then … / by wip(?goal)`). Incomplete.
#expect: Continue with:
import nat;
import core(symmetry);

lemma zero_left_flip(n : Nat)
  |- n = 0 + n;
proof
  by symmetry(Nat, 0 + n, n)?;
wip;
