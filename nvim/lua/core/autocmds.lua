-- Autocommands
-- Additional autocommands for modern Neovim

-- Highlight on yank
vim.api.nvim_create_autocmd("TextYankPost", {
  group = vim.api.nvim_create_augroup("HighlightYank", { clear = true }),
  callback = function()
    vim.highlight.on_yank({ higroup = "IncSearch", timeout = 200 })
  end,
})

-- Remove trailing whitespace on save
vim.api.nvim_create_autocmd("BufWritePre", {
  group = vim.api.nvim_create_augroup("TrimWhitespace", { clear = true }),
  pattern = "*",
  callback = function()
    local save_cursor = vim.fn.getpos(".")
    vim.cmd([[%s/\s\+$//e]])
    vim.fn.setpos(".", save_cursor)
  end,
})

-- Auto-resize splits when terminal is resized
vim.api.nvim_create_autocmd("VimResized", {
  group = vim.api.nvim_create_augroup("ResizeSplits", { clear = true }),
  callback = function()
    vim.cmd("wincmd =")
  end,
})

-- Go to last location when opening a buffer
vim.api.nvim_create_autocmd("BufReadPost", {
  group = vim.api.nvim_create_augroup("LastLocation", { clear = true }),
  callback = function()
    local mark = vim.api.nvim_buf_get_mark(0, '"')
    local lcount = vim.api.nvim_buf_line_count(0)
    if mark[1] > 0 and mark[1] <= lcount then
      pcall(vim.api.nvim_win_set_cursor, 0, mark)
    end
  end,
})

-- Check if buffer is git-tracked when opened (cache for performance)
vim.api.nvim_create_autocmd({"BufReadPost", "BufNewFile"}, {
  group = vim.api.nvim_create_augroup("CheckGitTracked", { clear = true }),
  callback = function()
    local filepath = vim.api.nvim_buf_get_name(0)
    if filepath ~= "" then
      local result = vim.fn.system("git ls-files --error-unmatch " .. vim.fn.shellescape(filepath) .. " 2>/dev/null")
      vim.b.is_git_tracked = (vim.v.shell_error == 0)
    else
      vim.b.is_git_tracked = false
    end
  end,
})

-- Auto-save git-tracked files after text changes (with debounce)
local save_timers = {}
local function debounced_save()
  local bufnr = vim.api.nvim_get_current_buf()
  
  -- Check if file is git-tracked (from cache) - early return for efficiency
  local is_tracked = vim.b[bufnr].is_git_tracked
  if not is_tracked then
    return
  end
  
  -- Cancel existing timer for this buffer
  if save_timers[bufnr] then
    vim.fn.timer_stop(save_timers[bufnr])
  end
  
  -- Set new timer (1000ms after last change)
  save_timers[bufnr] = vim.fn.timer_start(1000, function()
    -- Check if buffer still exists and is valid
    if not vim.api.nvim_buf_is_valid(bufnr) then
      save_timers[bufnr] = nil
      return
    end
    
    -- Check if file is modified and modifiable
    if not vim.bo[bufnr].modified or not vim.bo[bufnr].modifiable then
      return
    end
    
    -- Save the file
    vim.api.nvim_buf_call(bufnr, function()
      vim.cmd("silent! write")
    end)
    
    save_timers[bufnr] = nil
  end)
end

vim.api.nvim_create_autocmd({"TextChanged", "TextChangedI"}, {
  group = vim.api.nvim_create_augroup("AutoSaveGitFiles", { clear = true }),
  callback = debounced_save,
})
