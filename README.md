# Algae

Algae v2 is a parser-oriented proof and algebraic-specification language: typed
sorts, total operators, product/sum types, propositions, sequents, axioms,
inference rules, lemmas/theorems, theories, laws, models, and **explicit proof
trees**. This repository is a from-scratch Rust implementation of the toolchain.

The language is specified in [`lang-specs/spec.md`](lang-specs/spec.md); for a
gentler, example-driven introduction see the
[tutorial](lang-specs/tutorial.md).

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
| `verify`    | Elaborate, then run the proof checker over every obligation. |
| `fmt`       | Normalize operator glyphs (ASCII→Unicode by default, `--ascii` for the reverse), preserving all whitespace. |

Global flags: `--stdlib <dir>` (select a vendored stdlib), `-p/--project <path>`
(`algae.json` or its directory), `-q/--quiet`, `-v/--verbose`. `fmt` also takes
`--stdout` and `--check`.

```sh
# Verify the standard library (all proofs pass)
cargo run -p algae-cli -- verify algae/stdlib/v1/

# Convert a file's operators to Unicode in place
cargo run -p algae-cli -- fmt examples/app/main.alg
```

## How it works

The toolchain is split into three crates:

- **`algae-kernel`** — the environment-free core: parsing, elaboration, and
  type/proof checking. It depends only on a parser library and touches no
  threads, filesystem, or terminal (this is what makes it portable, e.g. to
  WASM).
- **`algae-cli`** — the command-line front end: file reading, module resolution,
  and terminal reporting on top of the kernel.
- **`algae-wasm`** — a `wasm-bindgen` wrapper that compiles the kernel to
  WebAssembly and exposes `check`/`format` to JavaScript, with the standard
  library embedded so `import`s resolve in the browser. Powers the docs site.

Pipeline: **Parse → Elaborate → IR (interned) → Check**.

- *Elaboration* resolves names/imports, kind/type-checks, and unfolds every
  axiom/rule/lemma into self-contained **proof steps**. Each step records its
  context, current goal, the inlined tactic, the tactic arguments, and the next
  goal(s) — so checking needs no further elaboration and no cross-lemma lookups.
- Operators are defined by equational axioms; the checker treats those axioms as
  definitional rewrite rules (sound, since axioms are assumed true) for
  definitional equality.

Proof checking has three phases: (1) read the steps from the elaborated tree;
(2) verify each step locally (`next_goal == tactic(current_goal, args)`,
recomputed — never trusted); (3) verify the parent/child goal linkage and that
leaves close their goal.

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

A **CodeMirror 6** editor lives in
[`editors/codemirror/`](editors/codemirror/): syntax highlighting, plus live
proof checking and inline error reporting driven by `algae-wasm`. It is the
editing surface embedded in the documentation site.

## Documentation site

An interactive documentation site is built from [`docs/`](docs/) with Sphinx.
Every `.alg` example on the site is a live CodeMirror editor: readers can edit
proofs and check them in the browser (the kernel runs as WebAssembly — nothing
is sent to a server). It reuses the prose from `lang-specs/` as the single
source of truth.

```sh
# Build the whole site locally (needs cargo + wasm-pack, node + npm, and the
# Python packages in docs/requirements.txt), then open docs/_build/html.
bash docs/build.sh
```

CI (`.github/workflows/ci.yml`, the `docs` job) builds the same site on every
push/PR and, on `main`, deploys it to the **`gh-pages`** branch. To publish it,
enable GitHub Pages once in the repository settings with source **"Deploy from a
branch: `gh-pages`"**.
