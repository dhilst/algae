# Inspecting a tactic that yields 2+ subgoals suggests a paste-able `cases`
# skeleton, one `case` per goal, each branch left as a named hole. Incomplete.
#expect: case ⊢ a = a;
import core(and_intro);

sort T : Sort;
op a : -> T;

lemma both
  |- a = a /\ a = a;
proof
  by and_intro(a = a, a = a)?;
wip;
