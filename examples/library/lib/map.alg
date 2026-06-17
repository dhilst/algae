# A reusable, parametric finite map from K to V. `get` takes a default,
# returned when the key is absent. This module is meant to be `include`d and
# instantiated with `with (K := …, V := …)`.
sort Map[K, V];

op empty : → Map[K, V];
op put : Map[K, V] × K × V → Map[K, V];
op get : Map[K, V] × K × V → V;   # get(m, key, default)

var m : Map[K, V];
var key : K;
var val, dflt : V;

axiom get_empty get(empty(), key, dflt) = dflt;
axiom get_put_same get(put(m, key, val), key, dflt) = val;
