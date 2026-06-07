# algae - Algebraic Specifications for AI-Assisted Development

A Claude Code plugin and Codex CLI skill for writing lightweight algebraic specifications using equational notation.

`.alg` files describe sorts, operation signatures, variables, and axioms. The parser checks syntax only; it does not perform type checking, model checking, or equational reasoning.

## Quick Example

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : -> Stack;
op push : Stack × Elem -> Stack;
op pop : Stack -> Stack | Error;
op top : Stack -> Elem | Error;

var s : Stack;
var e : Elem;

axiom push_top top(push(s, e)) = e;
axiom push_pop pop(push(s, e)) = s;
axiom empty_top top(empty()) = empty_error;
axiom empty_pop pop(empty()) = empty_error;
```

## Install

### Claude Code

```bash
claude plugin add /path/to/algae
```

### Codex CLI

```bash
./install-codex.sh
```

## Usage

| Command | Description |
|---------|-------------|
| `/alg write <description>` | Author a `.alg` spec from natural language or existing code |
| `/alg refine <file.alg>` | Iteratively refine a spec in dialogue with the model |
| `/alg impl <file.alg>` | Generate implementation code from a spec |
| `/alg verify <file.alg>` | Check code conformance against a spec |
| `/alg extract <source-files...>` | Reverse-engineer a spec from existing code |
| `python algae.py check <file.alg>` | Check `.alg` syntax |
| `python algae.py fmt <file.alg>` | Respell symbol aliases (Unicode ⇄ ASCII), preserving layout |
| `python algae.py print <file.alg>` | Print the parsed AST as JSON |

## Language Overview

- **Declarations**: `sort`, `op`, `var`, `axiom`, `lemma`
- **Operation signatures**: `op push : Stack × Elem -> Stack;`
- **Sum/error result types**: `Stack | Error`
- **Explicit narrowing**: `T | Error` never narrows to `T` implicitly; declare `op cast : (T | Error) -> T;` and wrap happy-path uses with `cast(...)` (convention)
- **Partial operations**: `op assert : T | Error -/-> T;` (`-/->` is ASCII for `⇸`) marks an op whose application carries a proof obligation; purely syntactic for now
- **Lemmas with proof sketches**: `lemma name expr; proof ... qed;` — parsed and formatted, not yet verified
- **ASCII aliases** available for Unicode symbols, such as `*`, `arrow`, `/\`, `\/`, `Nat`, `Bool`, `neq`, and `implies`
- **Single-file specs**: no `spec`, `import`, or `extends`

See [skills/alg/references/syntax.md](skills/alg/references/syntax.md) for the full language reference.

## License

Apache 2.0
