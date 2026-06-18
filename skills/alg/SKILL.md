---
name: alg
description: "Algebraic specification language (.alg files) for precise code development using equational notation. Activate when: (1) user mentions specifications, specs, .alg files, or algebraic specs, (2) .alg files exist in the project and user is implementing or reviewing related code, (3) user asks to design, specify, or formalize module behavior. Also a slash command: /alg write authors specs from natural language or code, /alg refine iteratively improves an existing spec in dialogue with the user, /alg impl generates conforming implementations, /alg verify checks code against specs, /alg extract reverse-engineers specs from existing code."
argument-hint: <write|refine|impl|verify|extract> [args...]
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# Algebraic Specifications (`.alg`)

`.alg` files are lightweight algebraic specifications. They describe sorts (with kinds), module parameters, operation signatures, equations, proof obligations, lemmas, and inference rules. `algae.py check` parses and type-checks them; there is no model checking, and proofs are structure-checked but not discharged.

## Quick Reference

```
sort Stack : Sort;
sort Elem : Sort;
sort Error : Sort;

op empty_error : → Error;

op empty : → Stack;
op push : Stack × Elem → Stack;
op pop : Stack → Stack × Elem | Error;
op top : Stack → Elem | Error;

eq push_top(s : Stack, e : Elem) top(push(s, e)) = e;
eq push_pop(s : Stack, e : Elem) pop(push(s, e)) = (s, e);
eq empty_top empty().top = empty_error;
eq empty_pop empty().pop = empty_error;
```

**Key conventions:**
- `sort Name : Sort;` declares a carrier sort; `sort List : Sort → Sort;` declares a sort constructor, applied as `List[Elem]`.
- `param T : Sort;` declares a module parameter, bound by `include … with (T := …)`.
- `op` declares operation signatures; an empty domain is written `op empty : → Stack;`. A **nullary op is a constant** used bare (`z`, `empty_error`).
- Named constructors are ordinary nullary ops (`op missing_key : → Error;`), not enum sorts.
- `|` in result types represents an algebraic sum/union, commonly for error alternatives.
- `eq name(binders) lhs = rhs;` is a trusted equation; the binders are its schematic variables (there are no top-level `var`s). The name is required and unique.
- `prop name(binders) lhs = rhs;` is a proof obligation, discharged at the `include` site that instantiates the module declaring it.
- `lemma name(binders) lhs = rhs;` optionally followed by a `proof ... qed;` block of `goal <state> by <tactic> therefore <state | done>;` steps. Tactics are `rewrite >`/`rewrite <` and `wip` (which end the step at `;`), and `apply rule(args) <case ... qed;>* therefore <state> qed;` — an apply is a subproof, so its `qed`/`wip` terminator comes after the step's `therefore` (a zero-premise rule is `apply reflexivity(Nat, z) therefore done qed;`). Lemmas are structure-checked, not discharged.
- `wip` ("work in progress") discharges a goal provisionally, to be finished later; it is viral, so any subproof (proof block, `case`, `apply`, or include `props` block) that uses it must be closed with `wip` instead of `qed`.
- `op f : T | Error ⇸ T;` (ASCII `-/->`) declares a partial operation, conventionally one that narrows a sum; `check` treats it like a total op for now.
- `rule` declares an inference rule with named premise `case … end;` blocks; sequent contexts (`⊢`) may carry typed variables and assumptions.
- `let name = expr;` at top level names a term shared by later declarations; `let ... in` scopes a binding inside one body. `let (a, b) = expr in ...` destructures a product (`_` binds unused components). Sum-typed values cannot be destructured.
- There are **no built-in numeric sorts** (`Nat`, `Int`, `Real` are ordinary identifiers), no number literals, and no built-in arithmetic. Numbers, if needed, are user-declared.
- Application sugar: `x.f(a)` reads as `f(x, a)` (pipe-first) and `x ▷ f(a)` (alias `|>`) as `f(a, x)` (pipe-last). Prefer `.` for object-oriented targets and `▷` for functional ones.
- Lowercase ASCII aliases such as `product`, `arrow`, `neq`, and `implies` parse as Unicode symbols, but prefer the Unicode symbols (`×`, `→`, `∧`, `∨`) when writing specs.
- See [references/syntax.md](references/syntax.md) for the full grammar.
- See [references/examples.md](references/examples.md) for example specs.

## When Working Near `.alg` Files

If activated while implementing or reviewing code (rather than via an explicit `/alg` subcommand):

1. Scan for `.alg` files that specify the module you are about to implement.
2. Read the sorts to identify domain concepts and error constructors.
3. Map each `op` to the implementation's public functions or methods.
4. Use `eq`/`prop`/`lemma` declarations as behavioral laws and test ideas.
5. Preserve the distinction between checking and proving: `check` validates syntax and types, but equations are not proved or model-checked.

When reviewing code against a spec, follow the conformance checks of the `verify` subcommand below.

# /alg - Subcommands

Parse `$ARGUMENTS` to determine the subcommand.

**Arguments:** $ARGUMENTS

## Subcommand: `write`

**Usage:** `/alg write <description or file path>`

Create or update an equational `.alg` specification.

**If given a natural language description:**
1. Identify the module's domain concepts, error cases, and public operations.
2. Declare concepts with `sort Name : Sort;` (or `sort C : Sort → …` for type constructors); declare named error constructors as nullary ops (`op missing_key : → Error;`).
3. Declare public operations with `op name : Domain → Codomain;`.
4. Use `|` in codomains for algebraic alternatives, such as `Value | Error`.
5. Write `eq` declarations for key equations, introducing variables as binders (`eq f(x : T) … = …;`).
6. Write one complete `.alg` file next to the code it specifies.

**If given source code:**
For thorough extraction from code, prefer `/alg extract`. This shortcut should:
1. Read the source file.
2. Extract domain sorts, error constructors, and public operations.
3. Infer operation signatures from arguments, return values, and failure cases.
4. Infer equations from simple tests, examples, reversible operations, or documented laws.
5. Write the `.alg` file.

**Style rules:**
- Prefer Unicode symbols (`×`, `→`, `∧`, `∨`) over ASCII aliases.
- Specs are equational: no set-theory notation (`∈`, `∪`, `∅`, set literals).
- Introduce variables as binders on each `eq`/`prop`/`lemma`; there are no top-level `var`s.
- Use nullary ops for named constructors, not enum sorts; there are no built-in numeric sorts.
- Keep specs concise and equational.
- Do not use `var`, `axiom`, enum sorts (`sort X = {…}`), parametric sort declarations (`sort X[T]`), or the state-machine keywords (`spec`, `state`, `init`, `pre`, `post`, `import`, `extends`).

## Subcommand: `refine`

**Usage:** `/alg refine <file.alg> [focus...]`

Iteratively refine an existing specification in dialogue with the user. The
user steers; each round proposes a small, reviewable set of improvements.

1. Read the `.alg` file and validate it with `python algae.py check <file.alg>`.
   If it has syntax errors, propose fixes and stop until they are resolved.
2. Build a model of the spec: sorts, operations split into constructors
   (operations that produce a sort) and observers (operations that inspect
   one), and which equations mention each operation.
3. Analyze the spec across these dimensions (narrow to `[focus...]` if given):

| Dimension | What to look for |
|-----------|------------------|
| Coverage | Operations appearing in no equation; observers not defined over each constructor |
| Error behavior | `\| Error` codomains whose error constructors never appear as an equation result |
| Consistency | Equations with the same left-hand side but different right-hand sides |
| Redundancy | Equations derivable from others; unused sorts or parameters |
| Readability | Deeply nested terms that would read better as a `let ... in` chain; setup chains repeated across equations that a top-level `let name = expr;` could share |
| Signatures | Domains/codomains the equations contradict or imply are missing |

4. Present numbered findings, each with a concrete before/after proposal.
   Distinguish facts (e.g. "`pop` has no equation for `empty()`") from
   assumptions about intended behavior, and flag the assumptions.
5. Ask the user which findings to apply. Also accept free-form refinement
   requests ("add an equation for revoking roles") as the next round's input.
6. Apply the chosen edits, re-run `python algae.py check`, and show the diff.
7. Repeat from step 3 until the user is satisfied or no findings remain.
8. Finish with `python algae.py fmt --inplace <file.alg>` and a one-paragraph
   summary of what changed across the session.

**Rules:**
- Never invent domain behavior silently; every guessed equation is presented as a
  question, not applied by default.
- Keep rounds small: 3-6 findings at a time, highest impact first.
- Follow the style rules from `/alg write`.

## Subcommand: `impl`

**Usage:** `/alg impl <file.alg> [--lang python|rust|typescript|go|java]`

Generate an implementation from a specification.

1. Read the `.alg` file.
2. Detect the target language from `--lang`, or infer it from the project.
3. Generate code:
   - `sort Name : Sort` -> nominal type, class, interface, or type alias.
   - `sort C : Sort → Sort` -> generic/parameterized type.
   - nullary error constructors (`op missing : → Error;`) -> enum or tagged error constructors.
   - `op name : A × B → C` -> public function/method signature.
   - `A | Error` -> result/union/exception-returning behavior appropriate for the language.
   - `eq` -> implementation laws and unit-test candidates.
4. Place generated code alongside the `.alg` file unless the user specifies otherwise.
5. Add a one-line generated-from comment.

**Mapping conventions by language:**

| `.alg` construct | Python | Rust | TypeScript |
|-----------------|--------|------|------------|
| `sort User : Sort` | class/NewType/protocol | struct/trait marker | interface/type |
| `sort List : Sort → Sort` | generic class | generic struct | generic type |
| nullary error ctors | `Enum` | `enum` | string union/enum |
| `op f : A × B → C` | function/method | `fn` | function/method |
| `A | Error` | union/exception/result object | `Result<A, Error>` | union/result object |
| `eq inverse f(g(x)) = x` | unit test/property test | test/property | test/property |

## Subcommand: `verify`

**Usage:** `/alg verify <file.alg> [source-files...]`

Check whether implementation code conforms to a specification.

1. Read the `.alg` file.
2. If source files are given, read them. Otherwise, look for implementation files with the same stem.
3. For each `op`, find the corresponding public function/method.
4. Check conformance:

| Spec element | Check |
|-------------|-------|
| `sort` | Corresponding type/class/enum/concept exists |
| `op` | Corresponding callable exists with compatible arity/result behavior |
| result `| Error` | Error/failure cases are represented |
| `eq` | Behavior is implemented or covered by tests |

5. Report findings in a table and summarize PASS/WARN/FAIL/SKIP counts.

## Subcommand: `extract`

**Usage:** `/alg extract <source-files...> [--out <file.alg>]`

Reverse-engineer an `.alg` specification from existing implementation code.

1. Read the source file(s). If multiple files are given, treat them as parts of one module.
2. Detect the language from file extensions and content.
3. Identify the public API boundary.
4. Extract constructs using this inverse mapping:

| Language construct | `.alg` construct |
|---|---|
| Domain class/interface/type alias | `sort Name : Sort` |
| Generic/parameterized type | `sort C : Sort → Sort` (+ `param`) |
| Enum, string literal union, named error constants | nullary ops (`op a : → Error;`) |
| Public method/function | `op name : Domain → Codomain` |
| Multi-argument function | `A × B → C` |
| Optional/result/error return | `C | Error` |
| Tests and documented laws | `eq ...` |

5. Infer equations from:
   - Tests and examples.
   - Round-trip operations, such as `decode(encode(x)) = x`.
   - Constructors followed by observers, such as `top(push(s, e)) = e`.
   - Error behavior, such as `top(empty()) = empty_error`.

6. Write the `.alg` file:
   - Use `--out` if given, otherwise place it alongside the source.
   - Prefer Unicode symbols.
   - Add a one-line extracted-from comment.

7. After writing, show a summary:

```
## Extraction Summary: stack.py -> stack.alg

| Construct | Count | Details |
|-----------|-------|---------|
| sort      | 3     | Stack, Elem, Error |
| op        | 5     | empty_error, empty, push, pop, top |
| eq        | 4     | top/push, pop/push, empty errors |
```

**Judgment calls:**
- When behavior is ambiguous, note it with a short `#` comment.
- If a source file is large, focus on the public API.
- Do not claim model checking or proof.
