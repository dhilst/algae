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

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
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
| `/alg impl <file.alg>` | Generate implementation code from a spec |
| `/alg verify <file.alg>` | Check code conformance against a spec |
| `python algae.py check <file.alg>` | Check `.alg` syntax |
| `python algae.py fmt <file.alg>` | Format `.alg` syntax |
| `python algae.py print <file.alg>` | Print the parsed AST as JSON |

## Language Overview

- **Declarations**: `sort`, `op`, `var`, `axiom`
- **Operation signatures**: `op push : Stack × Elem -> Stack;`
- **Sum/error result types**: `Stack | Error`
- **Lowercase ASCII keyword aliases** available for Unicode symbols, such as `product`, `arrow`, `neq`, and `implies`
- **Single-file specs**: no `spec`, `import`, or `extends`

See [skills/alg-spec/references/syntax.md](skills/alg-spec/references/syntax.md) for the full language reference.

## License

Apache 2.0
