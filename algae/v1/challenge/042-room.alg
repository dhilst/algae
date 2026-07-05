# The implication is given; feed it what it hungers for.
import core(implication_elim);

sort T : Sort;
op a : -> T;
op b : -> T;
axiom imp |- (a = a) => (b = b);
axiom ante |- a = a;

lemma banish
  |- b = b;
proof
  by implication_elim(a = a, b = b) cases
    case
      |- (a = a) => (b = b);
      by imp;
    qed;
    case
      |- a = a;
      by wip;
    wip;
  wip;
wip;
