# Example `.alg` Specifications

## Stack

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : -> Stack;
op push : Stack × Elem -> Stack;
op pop : Stack -> Stack | Error;
op top : Stack -> Elem | Error;

var s : Stack;
var e : Elem;

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
```

## Key-Value Store

```
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
```

## ASCII Aliases

```
sort Store, Key, Value;
sort Error = {missing_key};

op empty_store : arrow Store;
op put : Store product Key product Value arrow Store;
op get : Store product Key arrow Value | Error;

var s : Store;
var k : Key;
var v : Value;

axiom get(put(s, k, v), k) neq emptyset;
```
