-- Auto-registers the `alg` filetype when this directory is on the runtimepath,
-- so the bare `runtimepath:append(...)` setup needs nothing else.
vim.filetype.add({ extension = { alg = "alg" } })
