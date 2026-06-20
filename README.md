# Algae

Algae v2 is a parser-oriented proof and algebraic-specification language: typed
sorts, total operators, product/sum types, propositions, sequents, axioms,
inference rules, lemmas/theorems, theories, laws, models, and **explicit proof
trees**. This repository is a from-scratch Rust implementation of the toolchain.

The language is specified in [`lang-specs/spec.md`](lang-specs/spec.md).

## Building

Requires a Rust toolchain (`cargo`).

```sh
cargo build --release
```

## CLI

```
algae <command> [targets...] [flags]
```

| Command     | Description |
|-------------|-------------|
| `parse`     | Tokenize and parse; report syntax errors (`--dump-ast` to print the tree). |
| `typecheck` | Parse + elaborate (name/import resolution, kind/type checks, build proof steps). |
| `compile`   | Full pipeline → write `.algo` bytecode (parallel across files; cached). |
| `verify`    | Compile if needed, then run the parallel proof checker over the bytecode. |
| `fmt`       | Normalize operator glyphs (ASCII→Unicode by default, `--ascii` for the reverse), preserving all whitespace. |

Global flags: `--stdlib <dir>` (select a vendored stdlib), `-p/--project <path>`
(`algae.json` or its directory), `-j/--jobs <N>`, `--force` (ignore cache),
`-q/--quiet`, `-v/--verbose`. `fmt` also takes `--stdout` and `--check`.

```sh
# Verify the standard library (all proofs pass)
cargo run -- verify algae/stdlib/v1/

# Compile to bytecode, then re-verify from cache (fast second run)
cargo run -- compile algae/stdlib/v1/
cargo run -- verify  algae/stdlib/v1/      # "(cached)"

# Convert a file's operators to Unicode in place
cargo run -- fmt examples/app/main.alg
```

## How it works

Compilation: **Parse → Elaborate → IR (interned) → Bytecode (`.algo`) → write**.

- *Elaboration* resolves names/imports, kind/type-checks, and unfolds every
  axiom/rule/lemma into self-contained **proof steps**. Each step records its
  context, current goal, the inlined tactic, the tactic arguments, and the next
  goal(s) — so checking needs no further elaboration and no cross-lemma lookups.
- Operators are defined by equational axioms; the checker treats those axioms as
  definitional rewrite rules (sound, since axioms are assumed true) for
  definitional equality.

Proof checking has three phases: (1) read the steps from bytecode; (2) **in
parallel**, verify each step locally (`next_goal == tactic(current_goal, args)`,
recomputed — never trusted); (3) verify the parent/child goal linkage and that
leaves close their goal.

`.algo` files are invalidated by a stable content hash of the source and of each
dependency; compilation is deterministic (byte-identical regardless of `--jobs`).

## Standard library

`algae/stdlib/v1/`: `core`, `adt`, `monad`, `option`, `result`, `list`, `nat`,
`group`. `algae verify algae/stdlib/v1/` checks every proof.

## Projects (`algae.json`)

A project manifest lets modules be imported from elsewhere in the tree:

```json
{
  "name": "app",
  "version": 1,
  "sources": ["."],
  "dependencies": [{ "name": "geometry", "path": "../geometry" }],
  "stdlib": "../../algae/stdlib/v1"
}
```

See [`examples/app/`](examples/app/) for a worked cross-tree import.

## Editor support

Tree-sitter grammar and queries live in [`editors/tree-sitter/`](editors/tree-sitter/)
(the generated `parser.c` is committed). A self-contained Neovim sample config is
in [`editors/neovim/`](editors/neovim/):

```sh
nvim -u editors/neovim/init.lua algae/stdlib/v1/nat.alg
```

It registers the `alg` filetype and the local parser without touching your own
Neovim configuration.
