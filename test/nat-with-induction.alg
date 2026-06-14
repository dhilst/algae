sort Nat;

op 0 : → Nat;
op s : Nat → Nat;

op + : Nat × Nat → Nat;

var n, m : Nat;

axiom add_zero_left
  0 + m = m;

axiom add_succ_left
  s(n) + m = s(n + m);

rule induction(n, P) {
  case base : P(0),
  case step : P(n) ⊢ P(s(n))
              ──────────────
                  P(n);
};

lemma add_zero_right
  n + 0 = n;
proof
  by induction(n);

  case base;
    ──────────
    0 + 0 = 0;

    0 + 0;
    = 0 by add_zero_left(m := 0);
  qed;

  case step;
    ih : n + 0 = n;
    ───────────────
    s(n) + 0 = s(n);

    s(n) + 0;
    = s(n + 0) by add_succ_left(m := 0);
    = s(n) by ih;
  qed;

qed;
