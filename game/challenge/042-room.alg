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
  by wip;
wip;
