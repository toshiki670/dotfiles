# Modern Neovim Configuration

モダンなNeovim設定 with Lua + lazy.nvim

## 概要

このディレクトリには、Neovim (0.8+) 用のモダンなLua設定が含まれています。
従来のVimScript + dein.vim設定から、Lua + lazy.nvimへ完全移行しました。

### 主な特徴

- 🚀 **高速起動**: lazy.nvimによる最適化された遅延読み込み
- 🎨 **モダンUI**: Treesitter、Telescope、Neo-treeによる美しいインターフェース
- 🔍 **インテリジェント補完**: LSP + nvim-cmpによる強力な補完機能
- 📦 **簡単な管理**: Mason経由でLSPサーバーを自動インストール
- 🛠️ **保守性**: Lua設定で読みやすく保守しやすい

## 前提条件

### 必須

- **Neovim 0.8+** (0.9以上推奨)
- **ripgrep** - Telescopeの高速検索
- **fd** - Telescopeのファイル検索
- **Node.js** - LSPサーバー（ts_ls、yamlls、jsonls、bashlsなど）の実行に必要
- **Python** - LSPサーバー（pyrightなど）の実行に必要

### インストール方法

#### 1. 基本ツールのインストール

**macOS:**

```bash
brew install neovim ripgrep fd
```

**Arch Linux:**

```bash
sudo pacman -S neovim ripgrep fd
```

**Ubuntu/Debian:**

```bash
sudo apt install neovim ripgrep fd-find
```

#### 2. Node.jsとPythonのインストール（mise推奨）

[mise](https://mise.jdx.dev/)を使用すると、プロジェクトごとに異なるバージョンの言語ランタイムを管理できます。

**miseのインストール:**

```bash
brew install mise
```

**Node.jsとPythonのインストール:**

```bash
# Node.js（LTS版）
mise use -g node@lts

# Python（最新版）
mise use -g python@latest

# バージョンを確認
node --version
python --version
```

**特定のバージョンを指定する場合:**

```bash
# Node.js 20.x
mise use -g node@20

# Python 3.12
mise use -g python@3.12
```

**Note:** `-g`フラグはグローバルインストールを意味します。プロジェクトごとに異なるバージョンを使用したい場合は、プロジェクトディレクトリで`-g`なしで実行してください。

### オプション (推奨)

```bash
brew install lazygit  # Git TUI（Neo-treeから起動可能）
```

## ディレクトリ構造

```text
~/.config/nvim/              # (dot_config/nvim/ から配置)
├── init.lua                 # エントリーポイント
├── lua/
│   ├── core/
│   │   ├── options.lua      # 基本設定
│   │   ├── keymaps.lua      # キーマッピング
│   │   ├── autocmds.lua     # 自動コマンド
│   │   └── lazy.lua         # lazy.nvim設定
│   └── plugins/
│       ├── colorscheme.lua  # カラースキーム
│       ├── treesitter.lua   # シンタックスハイライト
│       ├── lualine.lua      # ステータスライン
│       ├── lsp.lua          # LSP設定
│       ├── cmp.lua          # 補完設定
│       ├── telescope.lua    # ファジーファインダー
│       ├── neo-tree.lua     # ファイルエクスプローラー
│       ├── git.lua          # Git統合
│       └── utilities.lua    # 便利なプラグイン
```

## 主要プラグイン

### コアプラグイン

| プラグイン | 説明 | 置き換え元 |
| --- | --- | --- |
| [lazy.nvim](https://github.com/folke/lazy.nvim) | プラグインマネージャー | dein.vim |
| [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) | LSP設定 | LanguageClient-neovim |
| [mason.nvim](https://github.com/williamboman/mason.nvim) | LSPサーバー管理 | - |
| [nvim-cmp](https://github.com/hrsh7th/nvim-cmp) | 補完エンジン | deoplete |
| [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) | シンタックス | 従来のsyntax |
| [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) | ファジーファインダー | denite.nvim |
| [neo-tree.nvim](https://github.com/nvim-neo-tree/neo-tree.nvim) | ファイルツリー | defx.nvim |
| [lualine.nvim](https://github.com/nvim-lualine/lualine.nvim) | ステータスライン | lightline.vim |
| [gitsigns.nvim](https://github.com/lewis6991/gitsigns.nvim) | Git変更表示 | vim-gitgutter |

### ユーティリティ

- [which-key.nvim](https://github.com/folke/which-key.nvim) - キーマップヘルプ
- [nvim-autopairs](https://github.com/windwp/nvim-autopairs) - 括弧自動閉じ
- [Comment.nvim](https://github.com/numToStr/Comment.nvim) - コメント操作
- [trouble.nvim](https://github.com/folke/trouble.nvim) - 診断リスト
- [vim-illuminate](https://github.com/RRethy/vim-illuminate) - 単語ハイライト

## キーマップ

### ウィンドウ/分割管理 (`s` prefix)

| キー | 動作 |
| --- | --- |
| `sw` | ウィンドウ切り替え |
| `sj/sk/sh/sl` | ウィンドウ移動 (下/上/左/右) |
| `sr` | 水平分割 |
| `sv` | 垂直分割 |
| `sq` | ウィンドウを閉じる |
| `sQ` | バッファを閉じる |
| `st` | 新しいタブ |
| `sm/sn` | タブ移動 (次/前) |

### Telescope (`<Space>d` prefix)

| キー | 動作 |
| --- | --- |
| `<Space>df` | ファイル検索 |
| `<Space>dg` | テキスト検索 (grep) |
| `<Space>db` | バッファ一覧 |
| `<Space>do` | ドキュメントシンボル |
| `<Space>dh` | ヘルプタグ |
| `<Space>dr` | 最近開いたファイル |

### Neo-tree

| キー | 動作 |
| --- | --- |
| `<Space>t` | Neo-treeトグル |
| `<Space>e` | 現在のファイルを表示 |

### LSP

| キー | 動作 |
| --- | --- |
| `gd` | 定義へジャンプ |
| `gr` | 参照を表示 |
| `gi` | 実装へジャンプ |
| `K` | ホバー情報 |
| `<leader>rn` | リネーム |
| `<leader>ca` | コードアクション |
| `[d/]d` | 前/次の診断 |

### Git (`<Space>g` prefix)

| キー | 動作 |
| --- | --- |
| `<Space>ga` | git add (Gwrite) |
| `<Space>gc` | git commit |
| `<Space>gr` | git read |
| `<leader>gs` | git status |
| `<leader>gd` | git diff |
| `]c/[c` | 次/前のhunk |
| `<leader>hs` | hunkをstage |
| `<leader>hp` | hunkをプレビュー |

### その他

| キー | 動作 |
| --- | --- |
| `<Esc><Esc>` | 検索ハイライト解除 |
| `gcc` | 行コメントトグル |
| `gc` (visual) | 選択範囲コメント |
| `<Space>ch` | チートシート表示 |

## LSPサーバー

### 新しいAPI（Neovim 0.11+）を使用

従来の`require('lspconfig').server_name.setup()`から、新しい`vim.lsp.config()`と`vim.lsp.enable()`に移行済み。

デフォルトでインストールされるLSPサーバー:

- **lua_ls** - Lua
- **ts_ls** - TypeScript/JavaScript
- **solargraph** - Ruby (要手動インストール: `gem install solargraph`)
- **rust_analyzer** - Rust
- **pyright** - Python
- **jsonls** - JSON
- **yamlls** - YAML
- **bashls** - Bash

追加のLSPサーバーは`:Mason`で簡単にインストールできます。

## 旧設定からの移行

### 主な変更点

1. **プラグインマネージャー**: dein.vim → lazy.nvim
2. **LSP**: LanguageClient-neovim → nvim-lspconfig + mason.nvim
3. **補完**: deoplete → nvim-cmp
4. **ファジーファインダー**: denite → telescope.nvim
5. **ファイルツリー**: defx → neo-tree
6. **設定言語**: VimScript → Lua

### 共存

旧設定（`vim/`）とこの設定は共存可能です。以下のコマンドで切り替えられます:

```bash
# 旧設定（VimScript + dein.vim）
vim

# 新設定（Lua + lazy.nvim）
nvim
```

## トラブルシューティング

### プラグインが正しくインストールされない

```vim
:Lazy sync
```

### LSPサーバーが動作しない

```vim
:Mason
:LspInfo
:checkhealth
```

### solargraph（Ruby LSP）のインストールエラー

solargraphはMason経由でのインストールが失敗することがあります。以下のコマンドで手動インストールしてください:

```bash
gem install solargraph
```

mise（またはrbenv）を使用している場合:

```bash
# miseの場合
mise use ruby@latest
gem install solargraph

# rbenvの場合
rbenv global 3.x.x
gem install solargraph
rbenv rehash
```

### 設定をリセット

```bash
rm -rf ~/.local/share/nvim
rm -rf ~/.local/state/nvim
rm -rf ~/.cache/nvim
```

## カスタマイズ

各プラグインの設定は`lua/plugins/`ディレクトリ内のファイルで管理されています。
変更を加えた後は、`:Lazy reload`または再起動で適用されます。

## 参考資料

- [Neovim Documentation](https://neovim.io/doc/)
- [lazy.nvim Documentation](https://github.com/folke/lazy.nvim)
- [kickstart.nvim](https://github.com/nvim-lua/kickstart.nvim)
- [awesome-neovim](https://github.com/rockerBOO/awesome-neovim)

## ライセンス

元の設定と同じライセンスが適用されます。
