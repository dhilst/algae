# Stack with a partial assert that narrows pop's fallible result, plus lemmas
# proved by explicit rewrite steps. Lemmas are checked; the rewrite steps are
# recorded but not discharged.
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;

op empty  : → Stack;
op push   : Stack × Elem → Stack;
op pop    : Stack → Stack × Elem | Error;

# A partial operation (⇸, ASCII -/->) narrows a sum to one of its branches.
op assert : Stack × Elem | Error ⇸ Stack × Elem;

op fst : (Stack × Elem) → Stack;
op snd : (Stack × Elem) → Elem;

eq push_pop(s : Stack, e : Elem) s.push(e).pop = (s, e);
eq empty_pop empty().pop = empty_error;
eq assert_elim(s : Stack, e : Elem) (s, e).assert = (s, e);
eq fst_pair(s : Stack, e : Elem) (s, e).fst = s;
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

lemma pop_stack(s : Stack, e : Elem)
  s.push(e).pop.assert.fst = s;
proof
  goal
    ⊢ s.push(e).pop.assert.fst = s
  by rewrite > push_pop(s, e) with (s.push(e).pop := (s, e))
  therefore
    ⊢ (s, e).assert.fst = s;

  goal
    ⊢ (s, e).assert.fst = s
  by rewrite > assert_elim(s, e) with ((s, e).assert := (s, e))
  therefore
    ⊢ (s, e).fst = s;

  goal
    ⊢ (s, e).fst = s
  by rewrite > fst_pair(s, e) with ((s, e).fst := s)
  therefore
    ⊢ s = s;
qed;
