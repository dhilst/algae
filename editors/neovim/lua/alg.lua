-- Reusable setup for Algae (.alg) tree-sitter highlighting using Neovim's
-- built-in tree-sitter (no external plugins).
--
-- From your own init.lua:
--     vim.opt.runtimepath:append("/path/to/algae/editors/neovim")
--     require("alg").setup()
--
-- `setup()` registers the `alg` filetype, (re)builds the parser from the
-- committed parser.c if needed, and registers it explicitly by path so a stale
-- `alg.so` elsewhere on the runtimepath cannot shadow it. The queries +
-- ftplugin are picked up from this directory.

local M = {}

local function mtime(p)
  local st = (vim.uv or vim.loop).fs_stat(p)
  return st and st.mtime and st.mtime.sec or nil
end

-- Resolve the key paths relative to this module
-- (.../editors/neovim/lua/alg.lua).
local function paths()
  local this = debug.getinfo(1, "S").source:sub(2) -- strip leading "@"
  local dir = vim.fn.fnamemodify(this, ":p:h:h") -- .../editors/neovim
  local repo = vim.fn.fnamemodify(dir, ":h:h") -- repo root
  local src = repo .. "/editors/tree-sitter/src"
  return {
    dir = dir,
    src = src,
    parser_c = src .. "/parser.c",
    so = dir .. "/parser/alg.so",
  }
end

--- Build editors/neovim/parser/alg.so from the committed parser.c if it is
--- missing or older than parser.c. Returns the `.so` path on success, else nil.
function M.build()
  local p = paths()
  local so_t, c_t = mtime(p.so), mtime(p.parser_c)
  if so_t and (not c_t or so_t >= c_t) then
    return p.so -- up to date
  end

  local cc = vim.fn.exepath("cc")
  if cc == "" then
    cc = vim.fn.exepath("gcc")
  end
  if cc == "" then
    vim.notify("[alg] no C compiler (cc/gcc) found to build the parser", vim.log.levels.ERROR)
    return nil
  end

  vim.fn.mkdir(vim.fn.fnamemodify(p.so, ":h"), "p")
  local out = vim.fn.system({ cc, "-o", p.so, "-shared", "-Os", "-fPIC", "-I", p.src, p.parser_c })
  if vim.v.shell_error ~= 0 then
    vim.notify("[alg] parser build failed:\n" .. out, vim.log.levels.ERROR)
    return nil
  end
  return p.so
end

--- Register the `alg` filetype, build the parser if needed, register it by
--- explicit path, and put this directory on the runtimepath so its queries +
--- ftplugin activate. Idempotent.
function M.setup()
  local p = paths()
  -- Prepend so our queries/ftplugin take precedence over any stale copies.
  if not vim.tbl_contains(vim.opt.runtimepath:get(), p.dir) then
    vim.opt.runtimepath:prepend(p.dir)
  end
  vim.filetype.add({ extension = { alg = "alg" } })
  local so = M.build()
  if so then
    -- Explicit path registration bypasses runtimepath parser shadowing.
    pcall(vim.treesitter.language.add, "alg", { path = so })
  end
end

return M
