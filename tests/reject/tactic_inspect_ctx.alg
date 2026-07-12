# Inspecting a tactic that introduces an eigenvariable: the suggested `then …`
# continuation must restate the introduced context (`x : T`), not drop it —
# otherwise the paste-able snippet would be an invalid (weakening) continuation.
# The suggestion lays the context out one entry per line, closing with the goal.
#expect: ⊢ x = x;
import core(forall_intro, refl);

sort T : Sort;

lemma all_refl
  |- forall (x : T) st x = x;
proof
  by forall_intro(T, _ = _)?;
wip;
