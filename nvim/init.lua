-- init.lua
--           []
--   __    ____ __ _ _  _ __ ___
--   \ \  / /| || ' ' \| '__/ __|
--    \ \/ / | || | | || | | (__
-- o   \__/  |_||_|_|_||_|  \___|
--
-- Modern Neovim configuration with Lua
-- Migrated from VimScript + dein.vim to Lua + lazy.nvim

-- Load core configuration
require("core.options")
require("core.keymaps")
require("core.autocmds")

-- Load lazy.nvim and plugins
require("core.lazy")
