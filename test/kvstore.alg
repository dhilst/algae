# kvstore.alg — Simple key-value store

spec KVStore[K, V]

  state
    data : K → V

  init
    data = ∅

  inv dom(data) ⊆ K

  op put(k : K, v : V)
    post data' = data ⊕ {k ↦ v}

  op get(k : K) → V
    pre  k ∈ dom(data)
    ret  data(k)

  op delete(k : K)
    pre  k ∈ dom(data)
    post data' = data \ {k ↦ data(k)}

  op has(k : K) → 𝔹
    ret  k ∈ dom(data)

  op size → ℕ
    ret  |dom(data)|

  op keys → ℘(K)
    ret  dom(data)

  prop put(k, v).then(get(k)).ret = v
  prop put(k, v).then(delete(k)).then(has(k)).ret = false
  prop put(k, v).then(size).ret ≥ 1
