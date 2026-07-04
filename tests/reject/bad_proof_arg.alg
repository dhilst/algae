# `rewrite_r`'s `eq` argument must prove `a = b` (here `z = o`), but `zz`
# only proves `z = z`. The proof-argument check must reject this.
import core(rewrite_r, refl);

sort Nat : Sort;

op z : -> Nat;
op o : -> Nat;

axiom zz
  |- z = z;

lemma bad
  |- o = o;
proof
  by rewrite_r(Nat, z, o, zz, lambda (x : Nat) st o = o)
  then
    |- o = o;
    by refl(Nat, o);
qed;
