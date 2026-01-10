-- Telescope configuration
-- Fuzzy finder (replaces denite.nvim)

return {
  {
    "nvim-telescope/telescope.nvim",
    cmd = "Telescope",
    version = false,
    dependencies = {
      "nvim-lua/plenary.nvim",
      {
        "nvim-telescope/telescope-fzf-native.nvim",
        build = "make",
      },
    },
    keys = {
      -- File navigation (replacing denite file commands)
      { "<Space>df", "<cmd>Telescope find_files<cr>", desc = "Find Files" },
      { "<Space>dg", "<cmd>Telescope live_grep<cr>", desc = "Live Grep" },
      { "<Space>db", "<cmd>Telescope buffers<cr>", desc = "Buffers" },
      { "<Space>do", "<cmd>Telescope lsp_document_symbols<cr>", desc = "Document Symbols" },
      { "<Space>dh", "<cmd>Telescope help_tags<cr>", desc = "Help Tags" },
      { "<Space>dr", "<cmd>Telescope oldfiles<cr>", desc = "Recent Files" },
      { "<Space>dw", "<cmd>Telescope grep_string<cr>", desc = "Grep Word" },
      { "<Space>dc", "<cmd>Telescope commands<cr>", desc = "Commands" },
      { "<Space>dk", "<cmd>Telescope keymaps<cr>", desc = "Keymaps" },
      { "<Space>ds", "<cmd>Telescope git_status<cr>", desc = "Git Status" },
      { "<Space>dt", "<cmd>Telescope git_commits<cr>", desc = "Git Commits" },
      -- LSP pickers (gd, gr, gi are defined in lsp.lua to avoid conflicts)
      { "<leader>ds", "<cmd>Telescope diagnostics<cr>", desc = "Diagnostics" },
    },
    config = function()
      local telescope = require("telescope")
      local actions = require("telescope.actions")

      telescope.setup({
        defaults = {
          prompt_prefix = " ",
          selection_caret = " ",
          path_display = { "truncate" },
          sorting_strategy = "ascending",
          layout_strategy = "horizontal",
          layout_config = {
            horizontal = {
              prompt_position = "top",
              preview_width = 0.55,
              results_width = 0.8,
            },
            vertical = {
              mirror = false,
            },
            width = 0.87,
            height = 0.80,
            preview_cutoff = 120,
          },
          file_ignore_patterns = {
            "node_modules",
            ".git/",
            "dist/",
            "build/",
            "target/",
            "vendor/",
            "%.lock",
          },
          mappings = {
            i = {
              ["<C-n>"] = actions.move_selection_next,
              ["<C-p>"] = actions.move_selection_previous,
              ["<C-c>"] = actions.close,
              ["<C-j>"] = actions.cycle_history_next,
              ["<C-k>"] = actions.cycle_history_prev,
              ["<C-q>"] = actions.smart_send_to_qflist + actions.open_qflist,
              ["<CR>"] = actions.select_default,
              ["<C-x>"] = actions.select_horizontal,
              ["<C-v>"] = actions.select_vertical,
              ["<C-t>"] = actions.select_tab,
              ["<C-u>"] = actions.preview_scrolling_up,
              ["<C-d>"] = actions.preview_scrolling_down,
            },
            n = {
              ["q"] = actions.close,
              ["<CR>"] = actions.select_default,
              ["<C-x>"] = actions.select_horizontal,
              ["<C-v>"] = actions.select_vertical,
              ["<C-t>"] = actions.select_tab,
              ["j"] = actions.move_selection_next,
              ["k"] = actions.move_selection_previous,
              ["<C-u>"] = actions.preview_scrolling_up,
              ["<C-d>"] = actions.preview_scrolling_down,
            },
          },
        },
        pickers = {
          find_files = {
            theme = "dropdown",
            previewer = false,
            hidden = true,
          },
          buffers = {
            theme = "dropdown",
            previewer = false,
            initial_mode = "normal",
            mappings = {
              i = {
                ["<C-d>"] = actions.delete_buffer,
              },
              n = {
                ["dd"] = actions.delete_buffer,
              },
            },
          },
          git_files = {
            theme = "dropdown",
            previewer = false,
          },
        },
        extensions = {
          fzf = {
            fuzzy = true,
            override_generic_sorter = true,
            override_file_sorter = true,
            case_mode = "smart_case",
          },
        },
      })

      -- Load extensions
      telescope.load_extension("fzf")
    end,
  },
}
