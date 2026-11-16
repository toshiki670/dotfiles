-- Utility plugins
-- Various helpful plugins for better editing experience

return {
	-- Auto pairs: Automatically close brackets, quotes, etc.
	-- Replaces lexima.vim
	{
		"windwp/nvim-autopairs",
		event = "InsertEnter",
		config = function()
			require("nvim-autopairs").setup({
				check_ts = true,
				ts_config = {
					lua = { "string" },
					javascript = { "template_string" },
				},
				disable_filetype = { "TelescopePrompt", "vim" },
				fast_wrap = {
					map = "<M-e>",
					chars = { "{", "[", "(", '"', "'" },
					pattern = string.gsub([[ [%'%"%)%>%]%)%}%,] ]], "%s+", ""),
					offset = 0,
					end_key = "$",
					keys = "qwertyuiopzxcvbnmasdfghjkl",
					check_comma = true,
					highlight = "Search",
					highlight_grey = "Comment",
				},
			})

			-- Integration with nvim-cmp
			local cmp_autopairs = require("nvim-autopairs.completion.cmp")
			local cmp = require("cmp")
			cmp.event:on("confirm_done", cmp_autopairs.on_confirm_done())
		end,
	},

	-- Comment: Easy commenting
	-- Replaces tcomment_vim
	{
		"numToStr/Comment.nvim",
		keys = {
			{ "gcc", mode = "n", desc = "Comment toggle current line" },
			{ "gc", mode = { "n", "o" }, desc = "Comment toggle linewise" },
			{ "gc", mode = "x", desc = "Comment toggle linewise (visual)" },
			{ "gbc", mode = "n", desc = "Comment toggle current block" },
			{ "gb", mode = { "n", "o" }, desc = "Comment toggle blockwise" },
			{ "gb", mode = "x", desc = "Comment toggle blockwise (visual)" },
		},
		config = function()
			require("Comment").setup({
				padding = true,
				sticky = true,
				ignore = "^$",
				toggler = {
					line = "gcc",
					block = "gbc",
				},
				opleader = {
					line = "gc",
					block = "gb",
				},
				extra = {
					above = "gcO",
					below = "gco",
					eol = "gcA",
				},
				mappings = {
					basic = true,
					extra = true,
				},
				pre_hook = nil,
				post_hook = nil,
			})
		end,
	},

	-- Which-key: Show available keybindings
	{
		"folke/which-key.nvim",
		event = "VeryLazy",
		config = function()
			vim.o.timeout = true
			vim.o.timeoutlen = 300
			require("which-key").setup({
				preset = "modern",
				plugins = {
					marks = true,
					registers = true,
					spelling = {
						enabled = false,
						suggestions = 20,
					},
					presets = {
						operators = true,
						motions = true,
						text_objects = true,
						windows = true,
						nav = true,
						z = true,
						g = true,
					},
				},
				win = {
					border = "rounded",
					height = { min = 4, max = 25 },
					padding = { 2, 2, 2, 2 },
					wo = {
						winblend = 0,
					},
				},
				layout = {
					width = { min = 20, max = 50 },
					spacing = 3,
				},
			})

			-- Register key groups (new format)
			require("which-key").add({
				{ "<leader>c", group = "code" },
				{ "<leader>d", group = "debug" },
				{ "<leader>f", group = "file/find" },
				{ "<leader>g", group = "git" },
				{ "<leader>h", group = "hunk" },
				{ "<leader>q", group = "quit/session" },
				{ "<leader>s", group = "search" },
				{ "<leader>t", group = "toggle" },
				{ "<leader>w", group = "workspace" },
				{ "<leader>x", group = "diagnostics/quickfix" },
				{ "<Space>d", group = "denite/telescope" },
				{ "<Space>g", group = "git" },
				{ "<Space>r", group = "rails" },
				{ "s", group = "split/window" },
			})
		end,
	},

	-- Indent blankline: Show indent guides
	{
		"lukas-reineke/indent-blankline.nvim",
		event = { "BufReadPost", "BufNewFile" },
		main = "ibl",
		opts = {
			indent = {
				char = "|",
				tab_char = "|",
			},
			scope = { enabled = false },
			exclude = {
				filetypes = {
					"help",
					"alpha",
					"dashboard",
					"neo-tree",
					"Trouble",
					"lazy",
					"mason",
					"notify",
					"toggleterm",
					"lazyterm",
				},
			},
		},
	},

	-- Vim-illuminate: Highlight same words under cursor
	{
		"RRethy/vim-illuminate",
		event = { "BufReadPost", "BufNewFile" },
		config = function()
			require("illuminate").configure({
				providers = {
					"lsp",
					"treesitter",
					"regex",
				},
				delay = 200,
				filetype_overrides = {},
				filetypes_denylist = {
					"dirvish",
					"fugitive",
					"neo-tree",
				},
				under_cursor = true,
				large_file_cutoff = 2000,
				large_file_overrides = nil,
				min_count_to_highlight = 1,
			})
		end,
	},

	-- Trouble: Pretty diagnostics, references, quickfix and location list
	{
		"folke/trouble.nvim",
		cmd = "Trouble",
		keys = {
			{ "<leader>xx", "<cmd>Trouble diagnostics toggle<cr>", desc = "Diagnostics (Trouble)" },
			{ "<leader>xw", "<cmd>Trouble diagnostics toggle<cr>", desc = "Workspace Diagnostics" },
			{ "<leader>xd", "<cmd>Trouble diagnostics toggle filter.buf=0<cr>", desc = "Document Diagnostics" },
			{ "<leader>xq", "<cmd>Trouble qflist toggle<cr>", desc = "Quickfix List" },
			{ "<leader>xl", "<cmd>Trouble loclist toggle<cr>", desc = "Location List" },
			{ "gR", "<cmd>Trouble lsp_references toggle<cr>", desc = "LSP References" },
		},
		opts = {
			keys = {
				["<tab>"] = "jump",
				["<c-x>"] = "jump_split",
				j = "next",
				k = "prev",
			},
		},
	},

	-- Vim-matchup: Better % matching
	-- Migrated from original config
	{
		"andymass/vim-matchup",
		event = { "BufReadPost", "BufNewFile" },
		config = function()
			vim.g.matchup_matchparen_offscreen = { method = "popup" }
		end,
	},

	-- Web devicons: File icons
	{
		"nvim-tree/nvim-web-devicons",
		lazy = true,
	},

	-- Plenary: Lua functions library (dependency for many plugins)
	{
		"nvim-lua/plenary.nvim",
		lazy = true,
	},
}
