# A hole in a proof-argument position reports the proposition it must prove.
#expect: needs a proof
import nat;
import core(refl, backward);

lemma step(k : Nat, ih := k + 0 = k)
  |- s(k) + 0 = s(k);
proof
  by backward(Nat, k + 0, k, ?eq, s(k) + 0 = s(_))?;
wip;
