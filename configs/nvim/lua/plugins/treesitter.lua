-- Treesitter configuration
-- Parsers and queries come from nvim-treesitter (main branch); highlighting,
-- folding and injections are provided by Neovim itself.
-- Installing parsers requires the tree-sitter CLI (`brew install tree-sitter-cli`).

local parsers = {
	"bash",
	"css",
	"diff",
	"fish",
	"git_rebase",
	"gitcommit",
	"gitignore",
	"go",
	"html",
	"javascript",
	"json",
	"lua",
	"markdown",
	"markdown_inline",
	"python",
	"query",
	"ruby",
	"rust",
	"toml",
	"tsx",
	"typescript",
	"vim",
	"vimdoc",
	"yaml",
}

return {
	{
		"nvim-treesitter/nvim-treesitter",
		lazy = false, -- nvim-treesitter does not support lazy-loading
		build = ":TSUpdate",
		config = function()
			require("nvim-treesitter").install(parsers)

			-- Neovim only starts treesitter highlighting by itself for a few bundled
			-- languages, so start it for every filetype that has a parser.
			vim.api.nvim_create_autocmd("FileType", {
				group = vim.api.nvim_create_augroup("TreesitterStart", { clear = true }),
				callback = function(args)
					local lang = vim.treesitter.language.get_lang(args.match)
					if lang and vim.treesitter.language.add(lang) then
						vim.treesitter.start(args.buf, lang)
					end
				end,
			})

			-- Treesitter-based folding, not folded by default
			vim.opt.foldmethod = "expr"
			vim.opt.foldexpr = "v:lua.vim.treesitter.foldexpr()"
			vim.opt.foldenable = false
		end,
	},
}
