-- Self-contained Neovim sample config for the Algae v2 (.alg) language.
--
-- Launch with:
--     nvim -u editors/neovim/init.lua <some-file>.alg
--
-- This config does NOT touch the user's real ~/.config/nvim. It:
--   * detects the `alg` filetype for *.alg files,
--   * registers Neovim's built-in tree-sitter at the LOCAL parser
--     (editors/neovim/parser/alg.so, built from the committed parser.c),
--   * installs the highlight queries from editors/neovim/queries/alg/,
--   * starts tree-sitter highlighting via the ftplugin.
--
-- Requirements:
--   * Neovim 0.11+ (uses vim.treesitter.language.add and vim.filetype.add).
--   * A C compiler (cc/gcc) the first time, to build parser/alg.so from the
--     committed editors/tree-sitter/src/parser.c if the .so is missing.

-- Resolve paths relative to THIS file so it works from any cwd. -----------
local this_file = debug.getinfo(1, "S").source:sub(2) -- strip leading "@"
local config_dir = vim.fn.fnamemodify(this_file, ":p:h") -- editors/neovim
local repo_root = vim.fn.fnamemodify(config_dir, ":h:h") -- repo root

local parser_so = config_dir .. "/parser/alg.so"
local parser_c = repo_root .. "/editors/tree-sitter/src/parser.c"
local parser_src_dir = repo_root .. "/editors/tree-sitter/src"

-- Make this directory's queries/ discoverable by vim.treesitter. ----------
-- vim.treesitter looks for queries under <rtp>/queries/<lang>/*.scm
vim.opt.runtimepath:prepend(config_dir)

-- Build the parser .so on demand if it is missing. -----------------------
local function ensure_parser()
  if vim.uv and vim.uv.fs_stat(parser_so) then
    return true
  elseif vim.loop and vim.loop.fs_stat(parser_so) then
    return true
  end

  local cc = vim.fn.exepath("cc")
  if cc == "" then
    cc = vim.fn.exepath("gcc")
  end
  if cc == "" then
    vim.notify(
      "[alg] No C compiler (cc/gcc) found; cannot build parser/alg.so",
      vim.log.levels.ERROR
    )
    return false
  end

  vim.fn.mkdir(config_dir .. "/parser", "p")
  local cmd = {
    cc, "-o", parser_so, "-shared", "-Os", "-fPIC",
    "-I", parser_src_dir, parser_c,
  }
  local out = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    vim.notify("[alg] Failed to build parser:\n" .. out, vim.log.levels.ERROR)
    return false
  end
  return true
end

-- Register the alg filetype for *.alg ------------------------------------
vim.filetype.add({
  extension = {
    alg = "alg",
  },
})

-- Register the tree-sitter language pointing at the local parser. --------
if ensure_parser() then
  vim.treesitter.language.add("alg", { path = parser_so })
end

-- A few sensible defaults so the sample is pleasant to use. --------------
vim.opt.termguicolors = true
vim.opt.number = true
pcall(vim.cmd.colorscheme, "default")

-- Highlighting itself is started per-buffer by ftplugin/alg.lua.
