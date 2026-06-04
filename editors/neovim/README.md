# alg.nvim

Neovim syntax highlighting for `.alg` algebraic specifications, powered by
tree-sitter. Works with Neovim's built-in tree-sitter support (0.9+); the
nvim-treesitter plugin is not required.

## Layout

```
editors/neovim/
├── tree-sitter-alg/      # grammar (grammar.js + generated src/parser.c)
├── queries/alg/          # highlight queries
├── plugin/alg.lua        # filetype detection + vim.treesitter.start
├── ftplugin/alg.lua      # buffer-local options (commentstring)
└── Makefile              # builds parser/alg.so from the committed parser.c
```

## Build

Compiling the parser only needs a C compiler (`src/parser.c` is committed):

```sh
make -C editors/neovim
```

This produces `editors/neovim/parser/alg.so`, which Neovim picks up from the
runtimepath automatically.

`make generate` regenerates `src/parser.c` after editing `grammar.js`
(requires the `tree-sitter` CLI).

## Install

### lazy.nvim (local clone)

```lua
{
  dir = '/path/to/algae/editors/neovim',
  name = 'alg.nvim',
  build = 'make',
  -- Load eagerly: the plugin registers the .alg filetype itself,
  -- so lazy-loading on `ft = 'alg'` would never trigger.
  lazy = false,
}
```

### Manual

Add the directory to your runtimepath in `init.lua`:

```lua
vim.opt.runtimepath:prepend('/path/to/algae/editors/neovim')
```

and run `make -C /path/to/algae/editors/neovim` once.

## Grammar notes

The grammar mirrors `algae/parser.py`: top-level `sort` / `op` / `var` /
`axiom` declarations, equational type expressions (`×`, `→`, `|`, `Seq[...]`),
and Pratt-style operator precedence for terms (including `if`/`then`/`else`
and `let`/`in`). Unicode symbols and their ASCII / keyword aliases
(`->`/`arrow`, `/\`/`and`, `Nat`/`ℕ`, ...) are interchangeable, as in the
reference parser.

One deliberate superset: `Seq[...]` parses as any `identifier [ type ]`, so
unknown constructors still produce a tree instead of an error node; the
highlight query only marks `Seq` as a builtin. Use `algae.py check` for real
validation.
