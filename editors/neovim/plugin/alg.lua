-- Filetype detection and tree-sitter highlighting for .alg specifications.

vim.filetype.add({ extension = { alg = 'alg' } })

vim.api.nvim_create_autocmd('FileType', {
  pattern = 'alg',
  group = vim.api.nvim_create_augroup('alg.treesitter', {}),
  callback = function(args)
    local ok = pcall(vim.treesitter.start, args.buf, 'alg')
    if not ok then
      vim.notify_once(
        'alg: tree-sitter parser not found; run `make` in editors/neovim to build it',
        vim.log.levels.WARN
      )
    end
  end,
})
