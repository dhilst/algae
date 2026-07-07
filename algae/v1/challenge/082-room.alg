sort T : Sort;

rule symmetry(T : Sort, x y : T)
  ⊢ x = y
  ────────────────────────
  ⊢ y = x
end;

rule transitivity(T : Sort, x y z : T)
  ⊢ x = y;
  ⊢ y = z
  ────────────────────────
  ⊢ x = z
end;

lemma sym(a b : T, h := a = b)
  ⊢ b = a;
proof
  by wip(?goal);
wip;

lemma trans(a b c : T, h1 := a = b, h2 := b = c)
  ⊢ a = c;
proof
  by wip(?goal);
wip;

lemma chain(a b c : T, h1 := a = b, h2 := c = b)
  ⊢ a = c;
proof
  by wip(?goal);
wip;
