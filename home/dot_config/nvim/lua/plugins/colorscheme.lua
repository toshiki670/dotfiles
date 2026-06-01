-- Colorscheme configuration
-- Follows the system light/dark appearance, matching Ghostty/Fish:
--   light = One Half Light (sonph/onehalf), dark = Ayu (Shatur/neovim-ayu)
--
-- Neovim detects the terminal background color via OSC 11 on startup and sets
-- 'background' accordingly (triggering OptionSet). We switch colorscheme on that
-- event, so launching nvim picks the theme matching the current macOS appearance.
-- Runtime switching while nvim stays open is intentionally not handled
-- (reopen nvim to re-detect the current appearance).

return {
	{
		"Shatur/neovim-ayu",
		dependencies = {
			-- One Half Light. Its colorschemes live under the repo's `vim/`
			-- subdirectory, which we add to 'runtimepath' in config below.
			"sonph/onehalf",
		},
		priority = 1000, -- Load before other plugins
		lazy = false,
		config = function()
			-- onehalf keeps its Vim colorschemes under a `vim/` subdirectory, which
			-- lazy.nvim does not add to 'runtimepath' automatically. Add it so that
			-- `colorscheme onehalflight` can be found.
			vim.opt.runtimepath:append(vim.fn.stdpath("data") .. "/lazy/onehalf/vim")

			require("ayu").setup({
				mirage = false, -- plain Ayu dark, matching Ghostty's "Ayu"
				terminal = true,
			})

			local group = vim.api.nvim_create_augroup("SystemBackgroundTheme", { clear = true })

			-- Keep the editor background transparent (Ghostty paints it, with blur).
			local function make_transparent()
				for _, hl in ipairs({
					"Normal",
					"NormalNC",
					"NormalFloat",
					"SignColumn",
					"EndOfBuffer",
				}) do
					vim.api.nvim_set_hl(0, hl, { bg = "none" })
				end
			end

			local function apply()
				if vim.o.background == "light" then
					pcall(vim.cmd.colorscheme, "onehalflight")
				else
					pcall(vim.cmd.colorscheme, "ayu-dark")
				end
				make_transparent()
			end

			-- Re-apply whenever 'background' changes: startup OSC 11 detection sets
			-- it (possibly after this config runs), and a manual `:set background=...`
			-- also flips the theme.
			vim.api.nvim_create_autocmd("OptionSet", {
				group = group,
				pattern = "background",
				callback = apply,
			})

			apply() -- apply immediately for the current 'background'
		end,
	},
}
