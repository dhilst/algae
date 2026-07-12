# Freshness rule 1: applying a rule whose premise assumes a proposition already
# in context does not reintroduce it (the existing assumption discharges it), so
# the continuation legitimately introduces no new variable.
rule assumption(P G : Prop)
  h := P ⊢ G
  ────────────────────────
  ⊢ G
end;

lemma foo(P : Prop)
  |- P;
proof
  by assumption(P, P)
  then
    h := P
    |- P;
  by assumption(P, P)
  then
    h := P
    |- P;
  by h;
qed;
