# Uses the lib::map module instantiated at Key → Val. The same operations are
# reached three ways: opened unqualified (`empty`, `put`), via the alias
# (`m::get`), and via the full path (`lib::map::get`).
include lib::map with (K := Key, V := Val);
alias m = lib::map;
open lib::map (empty, put);

sort Key : Sort;
sort Val : Sort;

eq lookup_after_put(key : Key, val dflt : Val)
  m::get(put(empty(), key, val), key, dflt) = val;
eq lookup_missing(key : Key, dflt : Val)
  lib::map::get(empty(), key, dflt) = dflt;
