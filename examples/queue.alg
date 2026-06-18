# A FIFO queue in data-first notation. get returns the queue without its head
# paired with that head; head and drop are the two projections of get.
sort Queue : Sort;
sort Elem : Sort;

op new  : → Queue;
op put  : Queue × Elem → Queue;
op get  : Queue × Elem → Queue × Elem;   # get(q, default)
op head : Queue × Elem → Elem;
op drop : Queue → Queue;

eq get_default(d : Elem) new().get(d) = (new(), d);
eq get_last(e d : Elem) new().put(e).get(d) = (new(), e);

eq get_fifo(q : Queue, e f d : Elem)
  q.put(e).put(f).get(d) =
    let (rest, x) = q.put(e).get(d) in (rest.put(f), x);

eq head_get(q : Queue, d : Elem) q.head(d) = let (_, x) = q.get(d) in x;
eq drop_get(q : Queue, d : Elem) q.drop = let (r, _) = q.get(d) in r;
