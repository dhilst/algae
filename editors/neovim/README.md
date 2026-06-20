# Algae v2 — Neovim sample config

A **self-contained** Neovim configuration that adds tree-sitter syntax
highlighting for the Algae v2 language (`.alg` files). It opts you in
explicitly and never touches your real `~/.config/nvim`.

## Quick start

From the repository root:

```sh
nvim -u editors/neovim/init.lua examples/anything.alg
```

(Any `.alg` file works; the standard-library samples extracted from the spec
live under `editors/tree-sitter/test/stdlib/`.)

That is all. On first launch the config builds the parser shared object from
the committed C source if it is missing, registers the `alg` filetype and the
tree-sitter language, and starts highlighting.

## What it does

`init.lua`:

1. Resolves all paths relative to itself, so it works from any working
   directory.
2. Prepends `editors/neovim/` to `runtimepath` so Neovim's built-in
   tree-sitter finds the highlight queries at
   `editors/neovim/queries/alg/highlights.scm`.
3. Registers `*.alg` as filetype `alg` (`vim.filetype.add`).
4. Builds `editors/neovim/parser/alg.so` from the committed parser source
   (`editors/tree-sitter/src/parser.c`) if the `.so` is not present, then
   registers it with `vim.treesitter.language.add('alg', { path = ... })`.

`ftplugin/alg.lua` runs per `alg` buffer and calls `vim.treesitter.start()`
to enable highlighting (plus `#`-comment settings and 2-space indentation).

## Prerequisites

- **Neovim 0.11+** (uses `vim.treesitter.language.add` and
  `vim.filetype.add`). Tested with 0.11.6.
- **A C compiler** (`cc` or `gcc`) — only needed the *first* time, to compile
  `parser/alg.so` from the committed `editors/tree-sitter/src/parser.c`. After
  that the prebuilt `.so` is reused.
- **No plugins** are required. nvim-treesitter is **not** needed.
- The tree-sitter CLI and npm are **only** needed if you want to regenerate
  the grammar — see `editors/tree-sitter/`. They are not needed to use this
  config.

## Rebuilding the parser by hand

If you change the grammar and regenerate `parser.c`, rebuild the `.so`:

```sh
cc -o editors/neovim/parser/alg.so -shared -Os -fPIC \
   -I editors/tree-sitter/src editors/tree-sitter/src/parser.c
```

Or simply delete `editors/neovim/parser/alg.so` and relaunch — `init.lua`
rebuilds it automatically.

If you also change the highlight queries in
`editors/tree-sitter/queries/highlights.scm`, copy them across:

```sh
cp editors/tree-sitter/queries/highlights.scm \
   editors/neovim/queries/alg/highlights.scm
```

## Verifying headlessly

```sh
nvim --headless -u editors/neovim/init.lua \
  editors/tree-sitter/test/stdlib/nat.alg \
  -c 'set ft?' \
  -c 'lua print(vim.treesitter.get_parser():lang())' \
  -c 'lua local b=vim.api.nvim_get_current_buf(); print("highlighter active: "..tostring(vim.treesitter.highlighter.active[b]~=nil))' \
  -c 'qa'
```

Expected output:

```
  filetype=alg
alg
highlighter active: true
```
