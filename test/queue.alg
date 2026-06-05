sort Queue, Elem;

op new  : → Queue;
op put  : Queue × Elem → Queue;
op get  : Queue × Elem → Queue × Elem;
op head : Queue × Elem → Elem;
op drop : Queue → Queue;
op size : Queue → ℕ;

var q       : Queue;
var e, f, d : Elem;

axiom new().size = 0;
axiom q.put(e).size = q.size + 1;

axiom new().get(d)        = (new(), d);
axiom new().put(e).get(d) = (new(), e);

axiom q.put(e).put(f).get(d) = 
      let (rest, x) = q.put(e).get(d) in (rest.put(f), x);

axiom q.head(d) = let (_, x) = q.get(d) in x;
axiom q.drop    = let (r, _) = q.get(d) in r;
