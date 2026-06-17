# Uses the lib::map module instantiated at Key → Val. The same operations are
# reached three ways: opened unqualified (`empty`, `put`), via the alias
# (`m::get`), and via the full path (`lib::map::get`).
include lib::map with (K := Key, V := Val);
alias m = lib::map;
open lib::map (empty, put);

sort Key, Val;

var key : Key;
var val, dflt : Val;

axiom lookup_after_put m::get(put(empty(), key, val), key, dflt) = val;
axiom lookup_missing lib::map::get(empty(), key, dflt) = dflt;
