-- Standalone Neovim sample config for the Algae v2 (.alg) language.
--
-- Try it without touching your own config:
--     nvim -u editors/neovim/init.lua <some-file>.alg
--
-- It just wires up the reusable `alg` module (lua/alg.lua), which registers the
-- `alg` filetype, builds the tree-sitter parser from the committed parser.c if
-- needed, and activates highlighting via this directory's queries + ftplugin.
--
-- To use this from your OWN init.lua instead, see editors/neovim/README.md.
--
-- Requirements: Neovim 0.11+ and a C compiler (cc/gcc) for the one-time build.

local this = debug.getinfo(1, "S").source:sub(2) -- strip leading "@"
local dir = vim.fn.fnamemodify(this, ":p:h") -- editors/neovim

-- Make lua/alg.lua requireable, then do the setup.
vim.opt.runtimepath:prepend(dir)
require("alg").setup()

-- Sample niceties so the standalone session is pleasant to use.
vim.opt.termguicolors = true
vim.opt.number = true
pcall(vim.cmd.colorscheme, "default")
