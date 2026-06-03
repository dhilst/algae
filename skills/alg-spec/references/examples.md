# Example `.alg` Specifications

## 1. Stack (basic generics, extends)

```
# stack.alg — LIFO container
import base/container

spec Stack[T] extends Container[T]

  state
    items : Seq[T]

  init
    items = []

  inv len(items) ∈ ℕ

  op push(x : T)
    post items' = items ++ [x]

  op pop → T
    pre  items ≠ []
    post items' = init(items)
    ret  last(items)

  op peek → T
    pre  items ≠ []
    ret  last(items)

  op size → ℕ
    ret  len(items)

  prop push(x).then(pop).ret = x
  prop push(x).then(size).ret = size.ret + 1
```

## 2. Auth (mappings, enums, records)

```
# auth.alg — Authentication module
import std/types

spec Auth

  type UserId = ℕ
  type Role = {admin, user, guest}
  type Cred = {user : UserId, pass : String}
  type Token = String
  type Store = UserId → Cred × Role

  state
    store  : Store
    active : UserId → Token

  init
    store  = ∅
    active = ∅

  inv dom(active) ⊆ dom(store)

  op register(c : Cred, r : Role)
    pre  c.user ∉ dom(store)
    post store' = store ∪ {c.user ↦ (c, r)}

  op login(uid : UserId, pass : String) → Token
    pre  uid ∈ dom(store) ∧ (store(uid)).pass = pass
    post ∃t ∈ Token · active' = active ∪ {uid ↦ t}
    ret  t

  op logout(uid : UserId)
    pre  uid ∈ dom(active)
    post active' = active \ {uid ↦ active(uid)}

  prop login(u, p).ret ≠ ∅ ⟹ u ∈ dom(active')
```

## 3. HTTP Router (function types, comprehensions)

```
# router.alg — HTTP request routing
import std/types

spec Router

  type Method = {GET, POST, PUT, DELETE, PATCH}
  type Path = String
  type Handler = Request → Response
  type Route = {method : Method, path : Path, handler : Handler}
  type Request = {method : Method, path : Path, headers : String → String, body : String}
  type Response = {status : ℕ, headers : String → String, body : String}

  state
    routes : Seq[Route]

  init
    routes = []

  inv ∀r ∈ routes · r.handler ∈ (Request → Response)

  fn match(req : Request) → Seq[Route] =
    {r ∈ routes | r.method = req.method ∧ r.path = req.path}

  op add(m : Method, p : Path, h : Handler)
    pre  ¬∃r ∈ routes · r.method = m ∧ r.path = p
    post routes' = routes ++ [{method : m, path : p, handler : h}]

  op dispatch(req : Request) → Response
    pre  match(req) ≠ []
    ret  head(match(req)).handler(req)

  op remove(m : Method, p : Path)
    pre  ∃r ∈ routes · r.method = m ∧ r.path = p
    post routes' = {r ∈ routes | ¬(r.method = m ∧ r.path = p)}
```

## 4. Event Bus (power sets, quantifiers)

```
# events.alg — Pub/sub event system
import std/types

spec EventBus[E]

  type SubId = ℕ
  type Callback = E → ()
  type Sub = {id : SubId, cb : Callback}

  state
    subs    : ℘(Sub)
    next_id : ℕ
    history : Seq[E]

  init
    subs    = ∅
    next_id = 0
    history = []

  inv ∀s ∈ subs · s.id < next_id
  inv |{s.id | s ∈ subs}| = |subs|   # ids are unique

  op subscribe(cb : Callback) → SubId
    post subs' = subs ∪ {{id : next_id, cb : cb}}
    post next_id' = next_id + 1
    ret  next_id

  op unsubscribe(sid : SubId)
    pre  ∃s ∈ subs · s.id = sid
    post subs' = {s ∈ subs | s.id ≠ sid}

  op publish(e : E)
    pre  true
    post history' = history ++ [e]
    post ∀s ∈ subs · s.cb(e)   # each subscriber is called

  op drain → Seq[E]
    post history' = []
    ret  history

  prop subscribe(cb).then(publish(e)) ⟹ cb(e)
```

## 5. Base Container (for extends)

```
# base/container.alg — Abstract container
spec Container[T]

  state
    elements : ℘(T)

  init
    elements = ∅

  op contains(x : T) → 𝔹
    ret  x ∈ elements

  op empty → 𝔹
    ret  elements = ∅
```
