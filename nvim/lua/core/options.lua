-- Core options
-- Migrated from vim/config/setting.vim

local opt = vim.opt
local g = vim.g

-- Disable compatibility with vi
vim.cmd("set nocompatible")

-- File encoding
opt.fileencoding = "utf-8"
opt.encoding = "utf-8"

-- Shell settings for Neovim
if vim.fn.has("unix") == 1 or vim.fn.has("mac") == 1 then
  opt.shell = "bash"
end

-- UI Settings
opt.showmode = false        -- Don't show mode (handled by statusline)
opt.laststatus = 2          -- Always show status line
opt.number = true           -- Show line numbers
opt.ruler = true            -- Show cursor position
opt.title = true            -- Show title
opt.showmatch = true        -- Show matching brackets
opt.ambiwidth = "double"    -- Double-width characters

-- Tab and indentation
opt.expandtab = true        -- Use spaces instead of tabs
opt.tabstop = 2             -- Tab width
opt.softtabstop = 2         -- Soft tab stop
opt.shiftwidth = 2          -- Indent width
opt.smartindent = true      -- Smart indentation

-- Invisible characters
opt.list = true
opt.listchars = {
  tab = "»-",
  trail = "-",
  eol = "↲",
  extends = "»",
  precedes = "«",
  nbsp = "%",
}

-- History
opt.history = 50

-- Cursor movement
opt.virtualedit = "block"   -- Free cursor in visual block mode
opt.whichwrap = "b,s,h,l,[,],<,>"  -- Wrap cursor at line boundaries

-- Relative line numbers (commented out in original)
-- opt.relativenumber = true

-- Backspace behavior
opt.backspace = { "indent", "eol", "start" }

-- Command mode
opt.showcmd = true          -- Show command in status line
opt.wildmenu = true         -- Command-line completion

-- Clipboard integration
if vim.fn.has("mac") == 1 then
  opt.clipboard:append("unnamed")
elseif vim.fn.has("unix") == 1 then
  opt.clipboard:append("unnamedplus")
end

-- Visual bell (no beep)
opt.visualbell = true

-- Buffer settings
opt.hidden = true           -- Allow hidden buffers

-- Search settings
opt.ignorecase = true       -- Ignore case in search
opt.smartcase = true        -- Case-sensitive if uppercase present
opt.incsearch = true        -- Incremental search
opt.wrapscan = true         -- Wrap search at end of file
opt.hlsearch = true         -- Highlight search results

-- Interactive substitution preview
opt.inccommand = "split"

-- Mouse support
if vim.fn.has("mouse") == 1 then
  opt.mouse = "a"           -- Enable mouse in all modes
end

-- JSON: Don't hide double quotes
vim.api.nvim_create_autocmd("FileType", {
  pattern = "json",
  callback = function()
    vim.opt_local.conceallevel = 0
  end,
})

-- Spell check (commented out in original)
-- opt.spell = true

-- Enable filetype plugins and indentation
vim.cmd("filetype plugin indent on")
