# algae - Algebraic Specifications for AI-Assisted Development

A Claude Code plugin and Codex CLI skill for writing lightweight algebraic specifications using equational notation.

`.alg` files describe sorts, module parameters, operation signatures, equations, proof obligations, lemmas, inference rules, and modules. `check` parses **and type-checks** them (kinds, operations, equations, lemma/prop/rule propositions, rule application, and module obligations). It does not perform model checking or equational/proof verification — proofs are parsed and their structure checked, but rewrite steps are not discharged.

## Quick Example

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
| `python algae.py check <file.alg>` | Check `.alg` syntax and types |
| `python algae.py fmt <file.alg>` | Respell symbol aliases (Unicode ⇄ ASCII), preserving layout |
| `python algae.py print <file.alg>` | Print the parsed AST as JSON |

## Language Overview

- **Declarations**: `sort`, `param`, `op`, `eq`, `prop`, `lemma`, `rule`, `include`, `open`, `alias`, `let`
- **Typed sorts (kinds)**: `sort Nat : Sort;`, sort constructors `sort List : Sort → Sort;`, used as `List[Elem]`
- **Module parameters**: `param T : Sort;` — abstract sorts/constructors bound by `include … with (…)`
- **Operation signatures**: `op push : Stack × Elem → Stack;` (nullary `→ T`, sum-typed `… | Error`, partial `⇸`). Nullary ops are constants, used bare (`z`, `empty_error`)
- **Equations**: `eq f(a : T) g(a) = a;` — a trusted equation; binder variables are its schematic parameters. There are no top-level `var`s and no built-in numeric sorts; sorts and their elements are user-declared
- **Proof obligations**: `prop name(x : T) lhs = rhs;` — required of any instantiation; discharged at the `include` site
- **Lemmas with proofs**: `lemma name(x : T) lhs = rhs; proof … qed;` — provable equations, parsed/structure-checked, not discharged
- **Sum/error result types**: `Stack × Elem | Error`
- **Explicit narrowing**: `T | Error` never narrows to `T` implicitly; declare `op cast : (T | Error) → T;` and wrap happy-path uses with `cast(...)` (convention)
- **Inference rules**: `rule` with named premise `case … end;` blocks; sequent contexts (`⊢`) may carry typed variables and assumptions
- **Structured proofs**: `goal <state> by <tactic> therefore <state | done>;`, with the `rewrite >`/`rewrite <`, `apply`, and `wip` tactics. Subproofs close with `qed`, or `wip` ("work in progress") when they use (virally) the `wip` tactic
- **Modules**: `include foo::bar with (T := Elem);`, `open`, and `alias`, resolved through an `alg-project.json` project root; included `prop`s become obligations discharged in an `include … props <case …>* qed;` block
- **ASCII aliases** available for Unicode symbols, such as `*`, `arrow`, `/\`, `\/`, `Bool`, `neq`, and `implies`

See [skills/alg/references/syntax.md](skills/alg/references/syntax.md) for the full language reference.

## License

Apache 2.0
