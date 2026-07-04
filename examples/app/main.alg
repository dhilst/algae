# Imports `point` from the `geometry` dependency (resolved via algae.json) and
# `refl` from the standard library.
import core(refl);

import point(
  Point,
  origin
);

lemma origin_reflexive
  |- origin = origin;
proof
  by refl(Point, origin);
qed;
