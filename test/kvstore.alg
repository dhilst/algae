sort Store, Key, Value;
sort Error = {missing_key};

op empty_store : -> Store;
op put : Store × Key × Value -> Store;
op get : Store × Key -> Value | Error;
op delete : Store × Key -> Store;
op has : Store × Key -> 𝔹;

var s : Store;
var k : Key;
var v : Value;

axiom get(put(s, k, v), k) = v;
axiom has(put(s, k, v), k) = true;
axiom get(empty_store(), k) = missing_key;
