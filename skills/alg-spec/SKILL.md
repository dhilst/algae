---
name: alg-spec
description: "Algebraic specification language (.alg files) for precise code development using set-theoretic notation. Activate when: (1) user mentions specifications, specs, .alg files, or algebraic specs, (2) .alg files exist in the project and user is implementing or reviewing related code, (3) user asks to design, specify, or formalize module behavior. This skill teaches how to read, write, and use .alg specifications."
version: 1.0.0
---

# Algebraic Specifications (`.alg`)

`.alg` files are lightweight algebraic specifications using set-theoretic notation. They describe *what* code should do — types as sets, operations as functions with pre/post conditions — without model checking. The model interprets them directly.

## Quick Reference

**Structure of a spec:**

```
import path/to/dep        # import another .alg file

spec Name[T] extends Base  # declare spec, optional generics and inheritance

  type Alias = ℕ           # type = named set
  type Status = {a, b, c}  # enumeration set
  type Map = K → V         # mapping (partial function)

  state                    # mutable state variables
    items : Seq[T]

  init                     # initial values
    items = []

  inv |items| ≤ 100        # invariant (always holds)

  op push(x : T)           # operation
    pre  |items| < 100     #   precondition
    post items' = items ++ [x]  #   postcondition (primed = post-state)

  op pop → T               # operation with return
    pre  items ≠ []
    post items' = init(items)
    ret  last(items)        #   return value

  fn helper(x : ℤ) → ℕ = if x ≥ 0 then x else -x  # pure function

  prop push(x).then(pop).ret = x  # property (not checked, documents intent)
```

**Key conventions:**
- `x'` = post-state of `x`; unmentioned state vars are unchanged (implicit frame)
- `#` comments
- `∅` = empty set/map, `dom(f)`/`ran(f)` = keys/values of mapping
- See [references/syntax.md](references/syntax.md) for full symbol table and grammar
- See [references/examples.md](references/examples.md) for complete example specs

## When Working Near `.alg` Files

1. **Before implementing**: scan for `.alg` files that specify the module you're about to implement. Use `find . -name "*.alg"` or `Glob("**/*.alg")`.
2. **Read the spec**: understand types, state, invariants, and operations.
3. **Map to code**:
   - `type` declarations → type definitions, enums, structs, classes
   - `state` → fields, instance variables, or module-level state
   - `inv` → assertions, validation logic, or type constraints
   - `op` with `pre` → input validation, guard clauses
   - `op` with `post` → the core logic ensuring the postcondition holds
   - `ret` → return value
   - `prop` → unit test assertions
4. **Preserve invariants**: every public method must maintain all `inv` clauses.
5. **Test from props**: `prop` declarations are test cases. Generate tests that verify them.

## When Reviewing Code Against a Spec

- Check that every `op` in the spec has a corresponding function/method.
- Check that `pre` conditions are enforced (via validation, exceptions, or type system).
- Check that `post` conditions are satisfied by the implementation.
- Check that `inv` clauses hold after every public operation.
- Flag operations present in code but absent from the spec (undocumented behavior).
