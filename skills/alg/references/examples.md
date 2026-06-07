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

axiom push_top top(push(s, e)) = e;
axiom push_pop pop(push(s, e)) = s;
axiom empty_top top(empty()) = empty_error;
axiom empty_pop pop(empty()) = empty_error;
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

axiom get_put get(put(s, k, v), k) = v;
axiom has_put has(put(s, k, v), k) = true;
axiom get_empty get(empty_store(), k) = missing_key;
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

axiom get_put get(put(s, k, v), k) = v;
```

## Partial Operations And Lemmas

```
sort Stack, Elem;
sort Error = {empty_error};

op push   : Stack × Elem → Stack;
op pop    : Stack → Stack × Elem | Error;
op assert : Stack × Elem | Error ⇸ Stack × Elem;
op snd    : (Stack × Elem) → Elem;

var s : Stack;
var e : Elem;

axiom push_pop s.push(e).pop = (s, e);
axiom assert_elim (s, e).assert = (s, e);
axiom snd_pair (s, e).snd = e;

lemma pop_top
  s.push(e).pop.assert.snd = e;
proof
  s.push(e).pop.assert.snd;
  = (s, e).assert.snd by push_pop;
  = (s, e).snd by assert_elim;
  = e by snd_pair;
qed;
```
