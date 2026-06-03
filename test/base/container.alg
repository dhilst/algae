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
