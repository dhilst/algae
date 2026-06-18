# A key-value store with a fallible get and a boolean membership test.
sort Store : Sort;
sort Key : Sort;
sort Value : Sort;
sort Error : Sort;

op missing_key : → Error;

op empty_store : → Store;
op put : Store × Key × Value → Store;
op get : Store × Key → Value | Error;
op delete : Store × Key → Store;
op has : Store × Key → 𝔹;

eq get_put(s : Store, k : Key, v : Value) get(put(s, k, v), k) = v;
eq has_put(s : Store, k : Key, v : Value) has(put(s, k, v), k) = true;
eq get_empty(k : Key) get(empty_store(), k) = missing_key;
