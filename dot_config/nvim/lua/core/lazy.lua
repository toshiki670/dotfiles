-- lazy.nvim setup
-- Bootstrap lazy.nvim plugin manager

local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"

-- Auto-install lazy.nvim if not present
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable",
    lazypath,
  })
end

-- Add lazy.nvim to runtime path
vim.opt.rtp:prepend(lazypath)

-- Load plugins from lua/plugins/ directory
-- Each file in lua/plugins/ should return a plugin spec or table of specs
require("lazy").setup("plugins", {
  defaults = {
    lazy = false, -- Plugins load on startup by default (override per-plugin)
    version = false, -- Use latest commit (can be overridden per plugin)
  },
  install = {
    missing = true, -- Auto-install missing plugins on startup
    colorscheme = { "monokai-pro" }, -- Try to load colorscheme on install
  },
  checker = {
    enabled = true, -- Check for plugin updates
    notify = false, -- Don't notify about updates
    frequency = 3600, -- Check every hour
  },
  change_detection = {
    enabled = true, -- Automatically check for config file changes
    notify = false, -- Don't notify when config changes
  },
  performance = {
    rtp = {
      -- Disable some rtp plugins we don't need
      disabled_plugins = {
        "gzip",
        "matchit",
        "matchparen",
        "netrwPlugin",
        "tarPlugin",
        "tohtml",
        "tutor",
        "zipPlugin",
      },
    },
  },
  ui = {
    border = "rounded",
    icons = {
      cmd = "⌘",
      config = "🛠",
      event = "📅",
      ft = "📂",
      init = "⚙",
      keys = "🗝",
      plugin = "🔌",
      runtime = "💻",
      source = "📄",
      start = "🚀",
      task = "📌",
      lazy = "💤",
    },
  },
})
