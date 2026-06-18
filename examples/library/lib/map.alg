# A reusable, parametric finite map from K to V. `get` takes a default,
# returned when the key is absent. This module is meant to be `include`d and
# instantiated with `with (K := …, V := …)`.
param K : Sort;
param V : Sort;

sort Map : Sort → Sort → Sort;

op empty : → Map[K, V];
op put : Map[K, V] × K × V → Map[K, V];
op get : Map[K, V] × K × V → V;   # get(m, key, default)

eq get_empty(key : K, dflt : V) get(empty(), key, dflt) = dflt;
eq get_put_same(m : Map[K, V], key : K, val dflt : V)
  get(put(m, key, val), key, dflt) = val;
