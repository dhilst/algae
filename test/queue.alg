sort Queue, Elem;

op new  : → Queue;
op put  : Queue × Elem → Queue;
op get  : Queue × Elem → Queue × Elem;
op head : Queue × Elem → Elem;
op drop : Queue → Queue;
op size : Queue → ℕ;

var q       : Queue;
var e, f, d : Elem;

axiom new_size new().size = 0;
axiom put_size q.put(e).size = q.size + 1;

axiom get_default new().get(d)        = (new(), d);
axiom get_last    new().put(e).get(d) = (new(), e);

axiom get_fifo q.put(e).put(f).get(d) =
      let (rest, x) = q.put(e).get(d) in (rest.put(f), x);

axiom head_get q.head(d) = let (_, x) = q.get(d) in x;
axiom drop_get q.drop    = let (r, _) = q.get(d) in r;
