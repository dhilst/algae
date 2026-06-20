# Imports `point` from the `geometry` dependency (resolved via algae.json) and
# `reflexivity` from the standard library.
import core(reflexivity);

import point(
  Point,
  origin
);

lemma origin_reflexive
  |- origin = origin;
proof
  by reflexivity(Point, origin);
qed;
