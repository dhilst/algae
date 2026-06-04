---
name: alg
description: "Write, implement, verify, and extract algebraic specifications (.alg files). Use /alg write to author specs from natural language or code, /alg impl to generate conforming implementations, /alg verify to check code against specs, /alg extract to reverse-engineer specs from existing code."
argument-hint: <write|impl|verify|extract> [args...]
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# /alg - Algebraic Specification Tool

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
- Keep the output syntax-only; do not claim model checking or proof.
