-- Filetype plugin for Algae v2 (.alg).
-- Runs for every buffer whose filetype is `alg`.

-- Comment string for `gcc` / commenting plugins (spec section 2.1: `#`).
vim.bo.commentstring = "# %s"
vim.bo.comments = ":#"

-- Reasonable indentation defaults.
vim.bo.expandtab = true
vim.bo.shiftwidth = 2
vim.bo.tabstop = 2
vim.bo.softtabstop = 2

-- Start tree-sitter highlighting. The parser + queries were registered by
-- init.lua; vim.treesitter.start uses the buffer's filetype -> language map.
-- Guard so a missing/unbuilt parser degrades gracefully instead of erroring.
local ok, err = pcall(vim.treesitter.start, 0, "alg")
if not ok then
  vim.notify("[alg] tree-sitter highlighting unavailable: " .. tostring(err),
    vim.log.levels.WARN)
end
