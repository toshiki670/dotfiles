-- Treesitter configuration
-- Better syntax highlighting and code understanding

return {
  {
    "nvim-treesitter/nvim-treesitter",
    version = false, -- Use latest commit
    build = ":TSUpdate",
    event = { "BufReadPost", "BufNewFile" },
    config = function()
      -- Minimal treesitter setup
      -- Parsers should be installed manually via :TSInstall or lazy.nvim build hook
      
      -- Enable treesitter-based folding
      vim.opt.foldmethod = "expr"
      vim.opt.foldexpr = "v:lua.vim.treesitter.foldexpr()"
      vim.opt.foldenable = false -- Don't fold by default
      
      -- Treesitter highlighting is enabled by default in Neovim 0.9+
      -- No additional configuration needed for basic functionality
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
