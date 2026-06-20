# `op` is a reserved keyword; using it as a term must be a parse error
# (keywords may not appear in term position).
sort Nat : Sort;

axiom bad
  ⊢ op = op;
