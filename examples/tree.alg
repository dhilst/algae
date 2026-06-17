# Binary trees over T: a `leaf` carries a value, a `node` joins two subtrees.
# `size` counts the leaves. The `tree_induction` rule is structural induction
# over trees; `size_positive` is proved with it. Proofs are structure-checked
# (each case states its instantiated subgoal); rewrite steps are not verified.
sort Tree[T];

op leaf : T → Tree[T];
op node : Tree[T] × Tree[T] → Tree[T];
op size : Tree[T] → ℕ;

var x : T;
var l, r, t : Tree[T];

axiom size_leaf size(leaf(x)) = 1;
axiom size_node size(node(l, r)) = size(l) + size(r);

# To prove P of every tree: prove it of every leaf, and of every node given
# that it holds of both subtrees.
rule tree_induction(l : Tree[T], r : Tree[T], P : Tree[T] → Prop)
  ⊢ ∀ (x : T) st P(leaf(x))
  P(l), P(r) ⊢ P(node(l, r))
  ─────────────────────────
  ⊢ ∀ (t : Tree[T]) st P(t)
end

lemma size_positive ⊢ ∀ (t : Tree[T]) st size(t) ≥ 1;
proof
  apply tree_induction(l, r, λ (t : Tree[T]) => size(t) ≥ 1);

  case [⊢ ∀ (x : T) st size(leaf(x)) ≥ 1]
    size(leaf(x));
    = 1 by size_leaf;
  qed;

  case [ihl := size(l) ≥ 1, ihr := size(r) ≥ 1 ⊢ size(node(l, r)) ≥ 1]
    size(node(l, r));
    = size(l) + size(r) by size_node;
  qed;
qed;
