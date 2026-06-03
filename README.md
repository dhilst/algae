# algae — Algebraic Specifications for AI-Assisted Development

A Claude Code plugin (and Codex CLI skill) for writing lightweight algebraic specifications using set-theoretic notation.

`.alg` files describe module behavior — types as sets, operations with pre/post conditions, invariants — in a token-efficient notation that leverages LLMs' set-theoretic reasoning. No model checking or equational reasoning; the AI interprets specs directly.

## Quick Example

```
spec Counter

  state
    value : ℤ

  init
    value = 0

  inv value ∈ ℤ

  op increment
    post value' = value + 1

  op decrement
    pre  value > 0
    post value' = value - 1

  op get → ℤ
    ret  value

  prop increment.then(decrement).then(get).ret = get.ret
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

The plugin also auto-activates when `.alg` files exist in your project — the model reads them to guide implementation and code review.

## Language Overview

- **13 keywords**: `spec` `extends` `import` `type` `state` `init` `inv` `op` `pre` `post` `ret` `prop` `fn`
- **Set-theoretic symbols**: `∈ ∉ ⊆ ⊂ ∪ ∩ × → ↦ ∅ ℕ ℤ ℝ 𝔹 ∀ ∃ ¬ ∧ ∨ ⟹ ℘`
- **ASCII fallbacks** available for all symbols
- **`#` comments**, UTF-8 encoding, `.alg` extension
- **Modularity**: `import path/name`, `spec Child extends Parent`

See [skills/alg-spec/references/syntax.md](skills/alg-spec/references/syntax.md) for the full language reference.

## License

MIT
