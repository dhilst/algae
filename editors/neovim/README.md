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

The grammar mirrors `algae/parser.py`. It covers:

- **Declarations**: `sort` (including parametric `sort List[T]`), `op`, `var`,
  `axiom`/`lemma` (with optional binder parameters and the `= prop` form),
  `rule`, `include` / `open` / `alias`, and top-level `let`.
- **Propositions**: sequents `assumptions ⊢ goal` with named assumptions
  (`h := A`), used by axioms, lemmas, and rule premises/conclusions.
- **Proofs**: `proof … qed`, rewrite steps (`= t by name;`), and `apply` with
  `case [..]` branches.
- **Types**: `×`, `→`/`⇸`, `|`, type application (`List[T]`, `Seq[T]`,
  `list::List[Elem]`), and qualified names.
- **Terms**: Pratt-style operator precedence, `if`/`then`/`else`, `let`/`in`,
  quantifiers (`∀ (n : ℕ) st …`, `∃`), and lambda (`λ (n : ℕ) => …` / `fun`).

Unicode symbols and their ASCII / keyword aliases (`->`/`arrow`, `/\`/`and`,
`Nat`/`ℕ`, `|-`/`⊢`, `forall`/`∀`, `fun`/`λ`, ...) are interchangeable, as in
the reference parser. The rule bar between premises and conclusion is a run of
the box-drawing dash `─` (U+2500). Use `algae.py check` for real validation
(type checking, module resolution); the grammar only describes shape.

Corpus tests live in `tree-sitter-alg/test/corpus/`; run them with
`tree-sitter test` from `tree-sitter-alg/`.
