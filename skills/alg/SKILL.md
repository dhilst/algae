---
name: alg
description: "Algebraic specification language (.alg files) for precise code development using equational notation. Activate when: (1) user mentions specifications, specs, .alg files, or algebraic specs, (2) .alg files exist in the project and user is implementing or reviewing related code, (3) user asks to design, specify, or formalize module behavior. Also a slash command: /alg write authors specs from natural language or code, /alg refine iteratively improves an existing spec in dialogue with the user, /alg impl generates conforming implementations, /alg verify checks code against specs, /alg extract reverse-engineers specs from existing code."
argument-hint: <write|refine|impl|verify|extract> [args...]
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# Algebraic Specifications (`.alg`)

`.alg` files are lightweight algebraic specifications. They describe sorts, operation signatures, variables, and axioms. `algae.py check` parses and type-checks them; there is no model checking or proof.

## Quick Reference

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : -> Stack;
op push : Stack × Elem -> Stack;
op pop : Stack -> Stack | Error;
op top : Stack -> Elem | Error;

var s : Stack;
var e : Elem;

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
```

**Key conventions:**
- `sort` declares carrier sets or enum-like constructor sets.
- `op` declares operation signatures; an empty domain is written `op empty : -> Stack;`.
- `|` in result types represents an algebraic sum/union, commonly for error alternatives.
- `var` declarations are read as implicitly universally quantified over all axioms.
- `axiom` gives equations or predicates that document intended behavior.
- `let name = expr;` at top level names a term shared by later axioms; `let ... in` scopes a binding inside one axiom.
- `let (a, b) = expr in ...` destructures a product-typed value; `_` binds unused components (each `_` is a fresh variable, valid only in patterns). Sum-typed values cannot be destructured.
- Application sugar: `x.f(a)` reads as `f(x, a)` (pipe-first) and `x ▷ f(a)` (alias `|>`) as `f(a, x)` (pipe-last). Prefer `.` when the spec targets object-oriented code and `▷` when it targets functional code.
- Lowercase ASCII aliases such as `product`, `arrow`, `neq`, and `implies` parse as Unicode symbols, but prefer the Unicode symbols (`×`, `→`, `∧`, `∨`) when writing specs.
- See [references/syntax.md](references/syntax.md) for the full grammar.
- See [references/examples.md](references/examples.md) for example specs.

## When Working Near `.alg` Files

If activated while implementing or reviewing code (rather than via an explicit `/alg` subcommand):

1. Scan for `.alg` files that specify the module you are about to implement.
2. Read the sorts to identify domain concepts and error constructors.
3. Map each `op` to the implementation's public functions or methods.
4. Use `var` and `axiom` declarations as behavioral laws and test ideas.
5. Preserve the distinction between checking and proving: `check` validates syntax and types, but axioms are not proved or model-checked.

When reviewing code against a spec, follow the conformance checks of the `verify` subcommand below.

# /alg - Subcommands

Parse `$ARGUMENTS` to determine the subcommand.

**Arguments:** $ARGUMENTS

## Subcommand: `write`

**Usage:** `/alg write <description or file path>`

Create or update an equational `.alg` specification.

**If given a natural language description:**
1. Identify the module's domain concepts, error cases, and public operations.
2. Declare concepts with `sort`, using enum sorts for named constructors such as errors.
3. Declare public operations with `op name : Domain -> Codomain;`.
4. Use `|` in codomains for algebraic alternatives, such as `Value | Error`.
5. Declare variables with `var`; they are read as implicitly universal over axioms.
6. Add `axiom` declarations for key equations and behavioral laws.
7. Write one complete `.alg` file next to the code it specifies.

**If given source code:**
For thorough extraction from code, prefer `/alg extract`. This shortcut should:
1. Read the source file.
2. Extract domain sorts, error constructors, and public operations.
3. Infer operation signatures from arguments, return values, and failure cases.
4. Infer equations from simple tests, examples, reversible operations, or documented laws.
5. Write the `.alg` file.

**Style rules:**
- Prefer Unicode symbols (`×`, `→`, `∧`, `∨`) over ASCII aliases.
- Specs are equational: no set-theory notation (`∈`, `∪`, `∅`, quantifiers, set literals).
- Keep specs concise and equational.
- Do not use `spec`, `state`, `init`, `pre`, `post`, `ret`, `prop`, `import`, or `extends`.

## Subcommand: `refine`

**Usage:** `/alg refine <file.alg> [focus...]`

Iteratively refine an existing specification in dialogue with the user. The
user steers; each round proposes a small, reviewable set of improvements.

1. Read the `.alg` file and validate it with `python algae.py check <file.alg>`.
   If it has syntax errors, propose fixes and stop until they are resolved.
2. Build a model of the spec: sorts, operations split into constructors
   (operations that produce a sort) and observers (operations that inspect
   one), variables, and which axioms mention each operation.
3. Analyze the spec across these dimensions (narrow to `[focus...]` if given):

| Dimension | What to look for |
|-----------|------------------|
| Coverage | Operations appearing in no axiom; observers not defined over each constructor |
| Error behavior | `\| Error` codomains whose error constructors never appear as an axiom result |
| Consistency | Axioms with the same left-hand side but different right-hand sides |
| Redundancy | Axioms derivable from others; unused sorts or variables |
| Readability | Deeply nested terms that would read better as a `let ... in` chain; setup chains repeated across axioms that a top-level `let name = expr;` could share |
| Signatures | Domains/codomains the axioms contradict or imply are missing |

4. Present numbered findings, each with a concrete before/after proposal.
   Distinguish facts (e.g. "`pop` has no axiom for `empty()`") from
   assumptions about intended behavior, and flag the assumptions.
5. Ask the user which findings to apply. Also accept free-form refinement
   requests ("add an axiom for revoking roles") as the next round's input.
6. Apply the chosen edits, re-run `python algae.py check`, and show the diff.
7. Repeat from step 3 until the user is satisfied or no findings remain.
8. Finish with `python algae.py fmt --inplace <file.alg>` and a one-paragraph
   summary of what changed across the session.

**Rules:**
- Never invent domain behavior silently; every guessed axiom is presented as a
  question, not applied by default.
- Keep rounds small: 3-6 findings at a time, highest impact first.
- Follow the style rules from `/alg write`.

## Subcommand: `impl`

**Usage:** `/alg impl <file.alg> [--lang python|rust|typescript|go|java]`

Generate an implementation from a specification.

1. Read the `.alg` file.
2. Detect the target language from `--lang`, or infer it from the project.
3. Generate code:
   - `sort Name` -> nominal type, class, interface, or type alias.
   - `sort Error = {a, b}` -> enum or tagged error constructors.
   - `op name : A × B -> C` -> public function/method signature.
   - `A | Error` -> result/union/exception-returning behavior appropriate for the language.
   - `axiom` -> implementation laws and unit-test candidates.
4. Place generated code alongside the `.alg` file unless the user specifies otherwise.
5. Add a one-line generated-from comment.

**Mapping conventions by language:**

| `.alg` construct | Python | Rust | TypeScript |
|-----------------|--------|------|------------|
| `sort User` | class/NewType/protocol | struct/trait marker | interface/type |
| `sort Error = {missing}` | `Enum` | `enum` | string union/enum |
| `op f : A × B -> C` | function/method | `fn` | function/method |
| `A | Error` | union/exception/result object | `Result<A, Error>` | union/result object |
| `axiom f(g(x)) = x` | unit test/property test | test/property | test/property |

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
| `axiom` | Behavior is implemented or covered by tests |

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
| Domain class/interface/type alias | `sort Name` |
| Enum, string literal union, named error constants | `sort Error = {a, b, c}` |
| Public method/function | `op name : Domain -> Codomain` |
| Multi-argument function | `A × B -> C` |
| Optional/result/error return | `C | Error` |
| Tests and documented laws | `axiom ...` |

5. Infer axioms from:
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
| op        | 4     | empty, push, pop, top |
| var       | 2     | s, e |
| axiom     | 4     | top/push, pop/push, empty errors |
```

**Judgment calls:**
- When behavior is ambiguous, note it with a short `#` comment.
- If a source file is large, focus on the public API.
- Do not claim model checking or proof.
