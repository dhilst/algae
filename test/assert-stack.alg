# Stack with a partial assert that narrows pop's fallible result, plus
# lemmas with proof sketches. Lemmas and proofs are parsed but not checked.
sort Stack, Elem;

sort Error = {empty_error};

op empty  : → Stack;
op push   : Stack × Elem → Stack;
op pop    : Stack → Stack × Elem | Error;

# A partial operation (⇸, ASCII -/->) narrows a sum to one of its branches.
op assert : Stack × Elem | Error ⇸ Stack × Elem;

op fst : (Stack × Elem) → Stack;
op snd : (Stack × Elem) → Elem;

var s : Stack;
var e : Elem;

axiom push_pop
  s.push(e).pop = (s, e);

axiom empty_pop
  empty().pop = empty_error;

axiom assert_elim
  (s, e).assert = (s, e);

axiom fst_pair
  (s, e).fst = s;

axiom snd_pair
  (s, e).snd = e;

lemma pop_top
  s.push(e).pop.assert.snd = e;
proof
  s.push(e).pop.assert.snd;
  = (s, e).assert.snd by push_pop;
  = (s, e).snd by assert_elim;
  = e by snd_pair;
qed;

lemma pop_stack
  s.push(e).pop.assert.fst = s;
proof
  s.push(e).pop.assert.fst;
  = (s, e).assert.fst by push_pop;
  = (s, e).fst by assert_elim;
  = s by fst_pair;
qed;
