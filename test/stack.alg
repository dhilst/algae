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
    post elements' = elements ∪ {x}

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
