-- LSP configuration using new vim.lsp.config API (Neovim 0.11+)
-- Migrated from legacy require('lspconfig') to vim.lsp.config
-- Reference: https://github.com/neovim/nvim-lspconfig

return {
  -- Mason: LSP server installer
  {
    "williamboman/mason.nvim",
    cmd = "Mason",
    keys = { { "<leader>cm", "<cmd>Mason<cr>", desc = "Mason" } },
    build = ":MasonUpdate",
    config = function()
      require("mason").setup({
        ui = {
          border = "rounded",
          icons = {
            package_installed = "✓",
            package_pending = "➜",
            package_uninstalled = "✗",
          },
        },
      })
    end,
  },

  -- Mason-lspconfig: Bridge between Mason and lspconfig
  {
    "williamboman/mason-lspconfig.nvim",
    dependencies = { "mason.nvim" },
    config = function()
      -- Build ensure_installed list based on available tools
      local ensure_installed = {
        "lua_ls",        -- Lua
        "ts_ls",         -- TypeScript/JavaScript
        -- "solargraph", -- Ruby (install manually via: gem install solargraph)
        "rust_analyzer", -- Rust
        "pyright",       -- Python
        "jsonls",        -- JSON
        "yamlls",        -- YAML
        "bashls",        -- Bash
      }

      -- Only add gopls if Go is installed
      if vim.fn.executable("go") == 1 then
        table.insert(ensure_installed, "gopls")
      end

      require("mason-lspconfig").setup({
        ensure_installed = ensure_installed,
        automatic_installation = true,
      })
    end,
  },

  -- LSP configuration using NEW API (vim.lsp.config + vim.lsp.enable)
  {
    "neovim/nvim-lspconfig",
    event = { "BufReadPre", "BufNewFile" },
    dependencies = {
      "mason.nvim",
      "mason-lspconfig.nvim",
    },
    config = function()
      local capabilities = require("cmp_nvim_lsp").default_capabilities()

      -- LSP handlers configuration
      vim.lsp.handlers["textDocument/hover"] = vim.lsp.with(vim.lsp.handlers.hover, {
        border = "rounded",
      })

      vim.lsp.handlers["textDocument/signatureHelp"] = vim.lsp.with(vim.lsp.handlers.signature_help, {
        border = "rounded",
      })

      -- Diagnostic configuration
      vim.diagnostic.config({
        virtual_text = {
          prefix = "●",
          source = "if_many",
        },
        signs = true,
        underline = true,
        update_in_insert = false,
        severity_sort = true,
        float = {
          border = "rounded",
          source = "always",
        },
      })

      -- Diagnostic signs
      local signs = { Error = " ", Warn = " ", Hint = " ", Info = " " }
      for type, icon in pairs(signs) do
        local hl = "DiagnosticSign" .. type
        vim.fn.sign_define(hl, { text = icon, texthl = hl, numhl = hl })
      end

      -- On attach function (keymaps and settings per buffer)
      local on_attach = function(client, bufnr)
        local opts = { noremap = true, silent = true, buffer = bufnr }

        -- Keymaps
        vim.keymap.set("n", "gD", vim.lsp.buf.declaration, opts)
        vim.keymap.set("n", "gd", vim.lsp.buf.definition, opts)
        vim.keymap.set("n", "K", vim.lsp.buf.hover, opts)
        vim.keymap.set("n", "gi", vim.lsp.buf.implementation, opts)
        vim.keymap.set("n", "<C-k>", vim.lsp.buf.signature_help, opts)
        vim.keymap.set("n", "<leader>wa", vim.lsp.buf.add_workspace_folder, opts)
        vim.keymap.set("n", "<leader>wr", vim.lsp.buf.remove_workspace_folder, opts)
        vim.keymap.set("n", "<leader>wl", function()
          print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
        end, opts)
        vim.keymap.set("n", "<leader>D", vim.lsp.buf.type_definition, opts)
        vim.keymap.set("n", "<leader>rn", vim.lsp.buf.rename, opts)
        vim.keymap.set({ "n", "v" }, "<leader>ca", vim.lsp.buf.code_action, opts)
        -- Use Telescope for references if available
        vim.keymap.set("n", "gr", function()
          require("telescope.builtin").lsp_references()
        end, opts)
        vim.keymap.set("n", "<leader>f", function()
          vim.lsp.buf.format({ async = true })
        end, opts)

        -- Diagnostic keymaps
        vim.keymap.set("n", "[d", vim.diagnostic.goto_prev, opts)
        vim.keymap.set("n", "]d", vim.diagnostic.goto_next, opts)
        vim.keymap.set("n", "<leader>e", vim.diagnostic.open_float, opts)
        vim.keymap.set("n", "<leader>q", vim.diagnostic.setloclist, opts)

        -- Highlight symbol under cursor
        if client.server_capabilities.documentHighlightProvider then
          vim.api.nvim_create_augroup("lsp_document_highlight", { clear = false })
          vim.api.nvim_clear_autocmds({ buffer = bufnr, group = "lsp_document_highlight" })
          vim.api.nvim_create_autocmd({ "CursorHold", "CursorHoldI" }, {
            group = "lsp_document_highlight",
            buffer = bufnr,
            callback = vim.lsp.buf.document_highlight,
          })
          vim.api.nvim_create_autocmd("CursorMoved", {
            group = "lsp_document_highlight",
            buffer = bufnr,
            callback = vim.lsp.buf.clear_references,
          })
        end
      end

      -- Common configuration for all servers
      local default_config = {
        capabilities = capabilities,
        on_attach = on_attach,
      }

      -- Configure LSP servers using NEW API: vim.lsp.config()
      -- This replaces the old lspconfig.server_name.setup() pattern

      -- Lua
      vim.lsp.config("lua_ls", vim.tbl_extend("force", default_config, {
        settings = {
          Lua = {
            diagnostics = {
              globals = { "vim" },
            },
            workspace = {
              library = vim.api.nvim_get_runtime_file("", true),
              checkThirdParty = false,
            },
            telemetry = {
              enable = false,
            },
          },
        },
      }))

      -- TypeScript/JavaScript
      vim.lsp.config("ts_ls", default_config)

      -- Ruby
      vim.lsp.config("solargraph", vim.tbl_extend("force", default_config, {
        settings = {
          solargraph = {
            diagnostics = true,
          },
        },
      }))

      -- Rust
      vim.lsp.config("rust_analyzer", vim.tbl_extend("force", default_config, {
        settings = {
          ["rust-analyzer"] = {
            checkOnSave = {
              command = "clippy",
            },
          },
        },
      }))

      -- Python
      vim.lsp.config("pyright", default_config)

      -- Go
      vim.lsp.config("gopls", vim.tbl_extend("force", default_config, {
        settings = {
          gopls = {
            analyses = {
              unusedparams = true,
            },
            staticcheck = true,
            gofumpt = true,
          },
        },
      }))

      -- JSON
      vim.lsp.config("jsonls", default_config)

      -- YAML
      vim.lsp.config("yamlls", default_config)

      -- Bash
      vim.lsp.config("bashls", default_config)

      -- Enable LSP servers using NEW API: vim.lsp.enable()
      -- This activates the configs for their respective filetypes
      vim.lsp.enable("lua_ls")
      vim.lsp.enable("ts_ls")
      vim.lsp.enable("solargraph")
      vim.lsp.enable("rust_analyzer")
      vim.lsp.enable("pyright")
      
      -- Only enable gopls if Go is installed
      if vim.fn.executable("go") == 1 then
        vim.lsp.enable("gopls")
      end
      
      vim.lsp.enable("jsonls")
      vim.lsp.enable("yamlls")
      vim.lsp.enable("bashls")
    end,
  },
}
