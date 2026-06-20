# tree-sitter-alg

A tree-sitter grammar for the **Algae v2** language (`.alg` files), translated
from `lang-specs/spec.md` (sections 2–3, with the standard library from
sections 5–12 used as test input).

## Contents

- `grammar.js` — the grammar (covers both ASCII and Unicode operator forms,
  spec section 2.6).
- `queries/highlights.scm` — highlight queries (standard capture groups).
- `src/` — the generated parser (`parser.c`, `grammar.json`,
  `node-types.json`, and `tree_sitter/*.h`). **Committed**, so consumers do not
  need the CLI to build the parser.
- `test/corpus/` — `tree-sitter test` corpus tests.
- `test/stdlib/` — the 8 standard-library modules extracted from the spec,
  used as full-file parse smoke tests.
- `tree-sitter.json` / `package.json` — grammar metadata. The tree-sitter CLI
  version is **pinned to `0.26.9`** (the version used to generate `src/`).

## Tooling

Pinned tree-sitter CLI: **0.26.9** (see `tree-sitter.json` →
`tree-sitter-cli`, and `package.json` → `devDependencies.tree-sitter-cli`).

```sh
tree-sitter --version   # tree-sitter 0.26.9
```

## Make targets

```sh
make generate       # regenerate src/parser.c (+ json + headers)
make test           # run tree-sitter corpus tests
make parse-stdlib   # parse every test/stdlib/*.alg, fail on any ERROR node
make parse FILE=... # parse a single file
make clean          # remove generated src/
```

## Status

All 8 standard-library modules (`core`, `adt`, `monad`, `option`, `result`,
`list`, `nat`, `group`) and the Unicode sample parse with **no ERROR or
MISSING nodes**, and all corpus tests pass.

## Editor integration

See `../neovim/` for a self-contained Neovim sample config that uses this
grammar.
