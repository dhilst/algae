# Binary trees over T: a `leaf` carries a value, a `node` joins two subtrees.
# `mirror` swaps every node's children. The `tree_induction` rule is structural
# induction over trees; its `step` premise carries the subtree variables and
# induction hypotheses in the sequent context. Proofs are structure-checked
# only — rewrite steps are not discharged.
param T : Sort;

sort Tree : Sort → Sort;

op leaf : T → Tree[T];
op node : Tree[T] × Tree[T] → Tree[T];
op mirror : Tree[T] → Tree[T];

eq mirror_leaf(x : T) mirror(leaf(x)) = leaf(x);
eq mirror_node(l r : Tree[T]) mirror(node(l, r)) = node(mirror(r), mirror(l));

# To prove P of every tree: prove it of every leaf, and of every node given
# that it holds of both subtrees.
rule tree_induction(P : Tree[T] → Prop)
  case base
    ⊢ ∀ (x : T) st P(leaf(x))
  end;
  case step
    l : Tree[T], r : Tree[T], P(l), P(r) ⊢ P(node(l, r))
  end;
  ─────────────────────────
  ⊢ ∀ (t : Tree[T]) st P(t)
end;

# Mirroring twice is the identity. Stated here; the inductive proof is omitted.
lemma mirror_involutive(t : Tree[T])
  mirror(mirror(t)) = t;
