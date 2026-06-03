---
name: alg
description: "Write, implement, and verify algebraic specifications (.alg files). Use /alg write to author specs from natural language or code, /alg impl to generate conforming implementations, /alg verify to check code against specs."
argument-hint: <write|impl|verify> [args...]
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# /alg — Algebraic Specification Tool

Parse `$ARGUMENTS` to determine the subcommand.

**Arguments:** $ARGUMENTS

## Subcommand: `write`

**Usage:** `/alg write <description or file path>`

Create or update a `.alg` specification.

**If given a natural language description:**
1. Identify the module's responsibility, its key types, and operations.
2. Model types as sets: enumerations → `{a, b, c}`, records → `{field : Type}`, collections → `Seq[T]` or `℘(T)`, lookups → `K → V`.
3. Define state variables.
4. Write invariants that must always hold.
5. For each operation: determine preconditions (when is this valid?), postconditions (what changes?), and return values.
6. Add `prop` clauses for key behavioral properties.
7. Write the `.alg` file next to the code it specifies (e.g. `auth.alg` alongside `auth.py`).

**If given source code (file path):**
1. Read the source file.
2. Extract types, state, and public operations.
3. Reverse-engineer preconditions from guard clauses, validation, and error handling.
4. Reverse-engineer postconditions from mutation logic.
5. Identify invariants from assertions, comments, or structural constraints.
6. Write the `.alg` file.

**Style rules:**
- Prefer Unicode symbols (`∈ ∪ ∩ → ∀ ∃ ∧ ∨`) over ASCII fallbacks.
- Keep specs concise — one screen per spec if possible.
- Use `#` comments sparingly, only for non-obvious design decisions.
- If the module depends on other specs, use `import`.
- If it refines a more abstract spec, use `extends`.

## Subcommand: `impl`

**Usage:** `/alg impl <file.alg> [--lang python|rust|typescript|go|java]`

Generate an implementation from a specification.

1. Read the `.alg` file. If it has `import` statements, read those too.
2. Detect the target language from `--lang`, or infer from the project (look at existing files, package.json, Cargo.toml, etc.).
3. Generate code:
   - `type` → type definitions, enums, structs, dataclasses
   - `state` → class fields or module state
   - `init` → constructor / initialization
   - `inv` → validation methods or assertions called after mutations
   - `op` → methods/functions:
     - `pre` → guard clause at method entry (raise/return error if violated)
     - `post` → the implementation body that achieves the postcondition
     - `ret` → return statement
   - `fn` → pure helper functions
   - `prop` → unit test stubs
4. Place generated code alongside the `.alg` file unless the user specifies otherwise.
5. Add a comment at the top: `# Generated from <name>.alg` (one line only).

**Mapping conventions by language:**

| `.alg` construct | Python | Rust | TypeScript |
|-----------------|--------|------|------------|
| `type Enum = {a,b}` | `Enum` class | `enum` | `type \| union` |
| `type Rec = {f:T}` | `@dataclass` | `struct` | `interface` |
| `K → V` | `dict[K,V]` | `HashMap<K,V>` | `Map<K,V>` |
| `Seq[T]` | `list[T]` | `Vec<T>` | `T[]` |
| `℘(T)` | `set[T]` | `HashSet<T>` | `Set<T>` |
| `pre` | `if not: raise` | `assert!` / `Result` | `if (!): throw` |
| `inv` | `_check_invariants()` | `fn invariant(&self)` | `private checkInv()` |

## Subcommand: `verify`

**Usage:** `/alg verify <file.alg> [source-files...]`

Check whether implementation code conforms to a specification.

1. Read the `.alg` file and resolve imports.
2. If source files are given, read them. Otherwise, look for implementation files with the same stem (e.g. `auth.alg` → `auth.py`, `auth.rs`, `auth.ts`, etc.).
3. For each `op` in the spec, find the corresponding function/method in the code.
4. Check conformance:

| Spec element | Check |
|-------------|-------|
| `type` | Corresponding type/class/struct exists |
| `state` | Fields/attributes exist with compatible types |
| `inv` | Invariant is enforced (assertions, validation, or structural guarantee) |
| `op` exists | Corresponding function/method exists |
| `pre` | Precondition is checked (guard clause, validation, type constraint) |
| `post` | Implementation logic achieves the postcondition |
| `ret` | Return value matches |
| `prop` | Test exists (or could be written) to verify the property |

5. Report findings in a table:

```
## Verification Report: auth.alg vs auth.py

| Element | Status | Notes |
|---------|--------|-------|
| type UserId | PASS | `UserId = int` |
| state store | PASS | `self.store: dict[int, ...]` |
| inv dom(active) ⊆ dom(store) | WARN | No explicit check — relies on login/logout logic |
| op register pre | PASS | `if uid in self.store: raise` |
| op register post | PASS | `self.store[uid] = ...` |
| op login pre | FAIL | Missing password check |
| prop ... | SKIP | No test found |
```

6. Summarize: total PASS/WARN/FAIL/SKIP counts, and list recommended fixes for FAILs.
