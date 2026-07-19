import core(forward, refl);

sort B : Sort;
op tt  : → B;
op ff  : → B;
op neg : B → B;

axiom neg_tt ⊢ neg(tt) = ff;
axiom neg_ff ⊢ neg(ff) = tt;

lemma dneg_tt
  ⊢ neg(neg(tt)) = tt;
proof
  by wip(?goal);
wip;

lemma dneg_ff
  ⊢ neg(neg(ff)) = ff;
proof
  by wip(?goal);
wip;

lemma tneg_tt
  ⊢ neg(neg(neg(tt))) = ff;
proof
  by wip(?goal);
wip;
