# Freshness rule 2: a newly-introduced hypothesis whose name is taken by a
# different proposition is renamed (`h` → `h0`), so both stay usable.
rule assumption(P G : Prop)
  h := P ⊢ G
  ────────────────────────
  ⊢ G
end;

lemma foo(P Q : Prop)
  |- P;
proof
  by assumption(P, P)
  then
    h := P
    |- P;
  by assumption(Q, P)
  then
    h := P;
    h0 := Q
    |- P;
  by h;
qed;
