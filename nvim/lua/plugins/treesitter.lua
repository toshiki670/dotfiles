-- Treesitter configuration
-- Better syntax highlighting and code understanding

return {
  {
    "nvim-treesitter/nvim-treesitter",
    version = false, -- Use latest commit
    build = ":TSUpdate",
    event = { "BufReadPost", "BufNewFile" },
    config = function()
      -- Simplified treesitter setup without using configs module
      -- This avoids compatibility issues with nvim-treesitter.configs
      
      -- List of parsers to ensure are installed
      local parsers = {
        "lua", "vim", "vimdoc", "query",
        "javascript", "typescript", "tsx",
        "json", "html", "css",
        "ruby", "rust", "python",
        "bash", "markdown", "markdown_inline",
        "yaml", "toml", "go", "dockerfile",
      }

      -- Install parsers
      local ok, ts_install = pcall(require, "nvim-treesitter.install")
      if ok then
        for _, parser in ipairs(parsers) do
          ts_install.update({ with_sync = false })(parser)
        end
      end

      -- Enable highlighting via vim.treesitter
      vim.treesitter.language.register("python", "python")
      vim.treesitter.language.register("lua", "lua")
      
      -- Enable treesitter-based folding
      vim.opt.foldmethod = "expr"
      vim.opt.foldexpr = "v:lua.vim.treesitter.foldexpr()"
      vim.opt.foldenable = false -- Don't fold by default
    end,
  },
  
  -- Treesitter textobjects (temporarily disabled due to compatibility issues)
  -- TODO: Re-enable after nvim-treesitter is stable
  -- {
  --   "nvim-treesitter/nvim-treesitter-textobjects",
  --   event = { "BufReadPost", "BufNewFile" },
  --   dependencies = { "nvim-treesitter/nvim-treesitter" },
  --   config = function()
  --     require("nvim-treesitter.configs").setup({
  --       textobjects = {
  --         select = {
  --           enable = true,
  --           lookahead = true,
  --           keymaps = {
  --             ["af"] = "@function.outer",
  --             ["if"] = "@function.inner",
  --             ["ac"] = "@class.outer",
  --             ["ic"] = "@class.inner",
  --           },
  --         },
  --         move = {
  --           enable = true,
  --           set_jumps = true,
  --           goto_next_start = {
  --             ["]m"] = "@function.outer",
  --             ["]]"] = "@class.outer",
  --           },
  --           goto_next_end = {
  --             ["]M"] = "@function.outer",
  --             ["]["] = "@class.outer",
  --           },
  --           goto_previous_start = {
  --             ["[m"] = "@function.outer",
  --             ["[["] = "@class.outer",
  --           },
  --           goto_previous_end = {
  --             ["[M"] = "@function.outer",
  --             ["[]"] = "@class.outer",
  --           },
  --         },
  --       },
  --     })
  --   end,
  -- },
}
