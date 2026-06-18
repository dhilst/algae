# Example `.alg` Specifications

## Stack

```
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Stack × Elem | Error;
op top : Stack → Elem | Error;

eq push_top(s : Stack, e : Elem) top(push(s, e)) = e;
eq push_pop(s : Stack, e : Elem) pop(push(s, e)) = (s, e);
eq empty_top empty().top = empty_error;
eq empty_pop empty().pop = empty_error;
```

## Key-Value Store

```
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
```

## ASCII Aliases

```
sort Store : Sort;
sort Key : Sort;
sort Value : Sort;
sort Error : Sort;

op missing_key : arrow Error;
op empty_store : arrow Store;
op put : Store product Key product Value arrow Store;
op get : Store product Key arrow Value | Error;

eq get_put (s : Store, k : Key, v : Value) get(put(s, k, v), k) = v;
```

## Parametric Module With Constructors

```
param T : Sort;

sort Option : Sort → Sort;

op none : → Option[T];
op some : T → Option[T];
op or_else : Option[T] × T → T;

eq or_else_some(x d : T) or_else(some(x), d) = x;
eq or_else_none(d : T) or_else(none(), d) = d;
```

## Functor Module (laws as obligations)

```
param T : Sort;
param F : Sort → Sort;

op id   : T → T;
op comp : (T → T) × (T → T) → T → T;   # comp(g, f) is g after f
op map  : (T → T) × F[T] → F[T];

prop functor_identity(x : F[T])
  map(id, x) = x;

prop functor_composition(f g : T → T, x : F[T])
  map(comp(g, f), x) = map(g, map(f, x));
```

## Monad Module (return/bind, laws as obligations)

```
param T : Sort;
param M : Sort → Sort;

op ret  : T → M[T];
op bind : M[T] × (T → M[T]) → M[T];

prop left_identity(x : T, f : T → M[T])
  bind(ret(x), f) = f(x);

prop right_identity(m : M[T])
  bind(m, ret) = m;

prop associativity(m : M[T], f g : T → M[T])
  bind(bind(m, f), g) = bind(m, λ (x : T) => bind(f(x), g));
```

## Partial Operations And Lemmas

```
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;

op push   : Stack × Elem → Stack;
op pop    : Stack → Stack × Elem | Error;
op assert : Stack × Elem | Error ⇸ Stack × Elem;
op snd    : (Stack × Elem) → Elem;

eq push_pop(s : Stack, e : Elem) s.push(e).pop = (s, e);
eq assert_elim(s : Stack, e : Elem) (s, e).assert = (s, e);
eq snd_pair(s : Stack, e : Elem) (s, e).snd = e;

lemma pop_top(s : Stack, e : Elem)
  s.push(e).pop.assert.snd = e;
proof
  goal
    ⊢ s.push(e).pop.assert.snd = e
  by rewrite > push_pop(s, e) with (s.push(e).pop := (s, e))
  therefore
    ⊢ (s, e).assert.snd = e;

  goal
    ⊢ (s, e).assert.snd = e
  by rewrite > assert_elim(s, e) with ((s, e).assert := (s, e))
  therefore
    ⊢ (s, e).snd = e;

  goal
    ⊢ (s, e).snd = e
  by rewrite > snd_pair(s, e) with ((s, e).snd := e)
  therefore
    ⊢ e = e;
qed;
```

## Induction Over A User-Declared Nat

```
sort Nat : Sort;

op z : → Nat;
op s : Nat → Nat;
op add : Nat × Nat → Nat;

eq add_zero_left(n : Nat) add(z, n) = n;
eq add_succ_left(n m : Nat) add(s(n), m) = s(add(n, m));

rule reflexivity(T : Sort, x : T)
  ─────────────────────────
  ⊢ x = x
end;

rule induction(P : Nat → Prop)
  case base
    ⊢ P(z)
  end;
  case step
    n : Nat, P(n) ⊢ P(s(n))
  end;
  ─────────────────────────
  ⊢ ∀ (n : Nat) st P(n)
end;
```

## `wip` Tactic (work in progress)

```
sort Nat : Sort;

op z : → Nat;
op add : Nat × Nat → Nat;

eq add_zero_left(n : Nat) add(z, n) = n;

# A goal left for later: the `wip` tactic marks the proof work-in-progress, so it
# is closed with `wip` instead of `qed` (the marker is viral).
lemma todo(n : Nat)
  add(n, z) = n;
proof
  goal
    ⊢ add(n, z) = n
  by wip
  therefore
    ⊢ add(n, z) = n;
wip;
```
