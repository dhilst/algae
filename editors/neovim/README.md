# Algae v2 — Neovim support

Tree-sitter syntax highlighting for the Algae v2 language (`.alg` files) using
Neovim's **built-in** tree-sitter. No plugins required (nvim-treesitter is *not*
needed). Requires **Neovim 0.11+** and a C compiler (`cc`/`gcc`) for a one-time
parser build.

## Try it standalone (no changes to your config)

```sh
nvim -u editors/neovim/init.lua algae/stdlib/v1/nat.alg
```

`-u` loads only this sample config (a clean sandbox). On launch it registers the
`alg` filetype, builds the parser from the committed C source if needed, and
starts highlighting.

## Use it from your own `init.lua`

The heavy lifting lives in a reusable module, `lua/alg.lua`. Two ways:

**A — `require` (recommended; auto-builds the parser):**

```lua
vim.opt.runtimepath:append("/path/to/algae/editors/neovim")
require("alg").setup()
```

`setup()` registers the `alg` filetype, (re)builds `parser/alg.so` from the
committed `parser.c` when it is missing or older than the source, registers it
explicitly by path, and activates the queries + ftplugin from this directory.

**B — `make` + runtimepath (no Lua logic):**

```sh
make -C /path/to/algae/editors/neovim       # builds parser/alg.so
```
```lua
vim.opt.runtimepath:prepend("/path/to/algae/editors/neovim")
```

`ftdetect/alg.lua` registers the filetype, and the prebuilt parser, queries and
ftplugin are discovered from the runtimepath. Use `prepend` (not `append`) so
this parser/queries win over any stale copy elsewhere on the runtimepath.

## Migrating from an earlier manual snippet

If you previously pasted a snippet that built a parser into
`~/.local/share/nvim/site/parser/alg.so`, delete that stale parser once — it was
built from an older grammar and can shadow the current one (causing
`Invalid node type "→"` query errors):

```sh
rm -f ~/.local/share/nvim/site/parser/alg.so
```

Then use approach A or B above. The new setup never writes to the `site`
directory.

## What it does

- `lua/alg.lua` — `setup()` / `build()`. Resolves paths relative to itself, so
  it works from any clone location.
- `ftdetect/alg.lua` — registers `*.alg` → filetype `alg`.
- `ftplugin/alg.lua` — per-buffer: `#`-comments, 2-space indent, and
  `vim.treesitter.start`.
- `queries/alg/highlights.scm` — highlight captures (kept in sync with
  `editors/tree-sitter/queries/highlights.scm`).
- `parser/alg.so` — the compiled parser (git-ignored; built on demand).
- `init.lua` — the standalone `-u` entry point; just calls `require("alg").setup()`.

Both ASCII and Unicode operator spellings are highlighted (`|-`/`⊢`, `->`/`→`,
`/\`/`∧`, `\/`/`∨`, `=>`/`⇒`, `<=>`/`⇔`, `~`/`¬`, `lambda`/`λ`, and the
`------`/`──────` separator).

## After changing the grammar

Regenerate and rebuild (the `require` setup rebuilds automatically via an mtime
check; otherwise):

```sh
cd editors/tree-sitter && tree-sitter generate          # if you edited grammar.js
make -C editors/neovim                                   # rebuild parser/alg.so
cp editors/tree-sitter/queries/highlights.scm \
   editors/neovim/queries/alg/highlights.scm             # if you edited queries
```

## Verify headlessly

```sh
nvim -u editors/neovim/init.lua algae/stdlib/v1/nat.alg \
  -c 'lua print("lang="..vim.treesitter.get_parser(0):lang()..
       " ok="..tostring(not vim.treesitter.get_parser(0):parse()[1]:root():has_error()))' \
  -c 'qa'
```
