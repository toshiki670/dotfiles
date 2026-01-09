-- Keymaps
-- Migrated from vim/config/mapping.vim

local keymap = vim.keymap.set
local opts = { noremap = true, silent = true }

-- Escape terminal mode with ESC
keymap("t", "<ESC>", "<C-\\><C-n>", opts)

-- ESC ESC to clear search highlight
keymap("n", "<Esc><Esc>", ":nohlsearch<CR><Esc>", opts)

-- Center screen on search navigation
keymap("n", "n", "nzz", opts)
keymap("n", "N", "Nzz", opts)

-- Search and replace shortcut
keymap("n", "<Space>na", "*:%s///g<LEFT><LEFT>", { noremap = true })

-- Auto-insert brackets (commented out in original, handled by autopairs plugin)
-- keymap("i", "{<Enter>", "{}<Left><CR><ESC><S-o>", opts)
-- keymap("i", "[", "[]<LEFT>", opts)
-- keymap("i", "(", "()<LEFT>", opts)
-- keymap("i", "'", "''<LEFT>", opts)
-- keymap("i", '"', '""<LEFT>', opts)

-- Command mode shortcuts (commented out in original)
-- keymap("n", ";", ":", { noremap = true })
-- keymap("n", ":", ";", { noremap = true })

-- Window/Split management with 's' prefix
-- Disable default 's' behavior
keymap("n", "s", "<Nop>", opts)

-- Window navigation
keymap("n", "sw", "<C-w>w", opts)    -- Cycle through windows
keymap("n", "sj", "<C-w>j", opts)    -- Move down
keymap("n", "sk", "<C-w>k", opts)    -- Move up
keymap("n", "sl", "<C-w>l", opts)    -- Move right
keymap("n", "sh", "<C-w>h", opts)    -- Move left

-- Move window itself
keymap("n", "sJ", "<C-w>J", opts)
keymap("n", "sK", "<C-w>K", opts)
keymap("n", "sL", "<C-w>L", opts)
keymap("n", "sH", "<C-w>H", opts)

-- Tab navigation
keymap("n", "sm", "gt", opts)        -- Next tab
keymap("n", "sn", "gT", opts)        -- Previous tab

-- Resize windows
keymap("n", "s=", "<C-w>=", opts)    -- Equalize window sizes
keymap("n", "so", "<C-w>_<C-w>|", opts)  -- Maximize current window

-- Buffer navigation
keymap("n", "sN", ":<C-u>bn<CR>", opts)
keymap("n", "sP", ":<C-u>bp<CR>", opts)

-- New tab
keymap("n", "st", ":<C-u>tabnew<CR>", opts)

-- Tab list (will be handled by telescope)
-- keymap("n", "sT", ":<C-u>Unite tab<CR>", opts)

-- Split windows
keymap("n", "sr", ":<C-u>sp<CR>", opts)   -- Horizontal split
keymap("n", "sv", ":<C-u>vs<CR>", opts)   -- Vertical split

-- Close window/buffer
keymap("n", "sq", ":<C-u>q<CR>", opts)    -- Close window
keymap("n", "sQ", ":<C-u>bd<CR>", opts)   -- Close buffer

-- Buffer navigation (alternative, commented out in original)
-- keymap("n", "sm", ":bn<CR>", opts)
-- keymap("n", "sn", ":bp<CR>", opts)
