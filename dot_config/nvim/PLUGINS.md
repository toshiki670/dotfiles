# Neovim プラグイン一覧

現在インストールされているプラグインの完全なリストと、各プラグインの役割を説明します。

## 📦 プラグイン管理 (必須)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| [lazy.nvim](https://github.com/folke/lazy.nvim) | プラグインマネージャー | ⭐⭐⭐ 必須 | ❌ 削除不可 |

---

## 🎨 見た目・UI (一部必須)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **monokai-pro.nvim** | カラースキーム（Monokai Pro） | ⭐⭐ 推奨 | ✅ 他のテーマに変更可 |
| **lualine.nvim** | ステータスライン | ⭐⭐⭐ 必須 | ⚠️ 削除すると見た目が貧弱に |
| **nvim-web-devicons** | アイコン表示 | ⭐⭐⭐ 必須 | ❌ 多くのプラグインが依存 |
| **indent-blankline.nvim** | インデントガイド表示 | ⭐ オプション | ✅ 削除可 |

**推奨**: lualineとdeviconsは残す。indent-blanklineは好みに応じて。

---

## 🔍 ファジーファインダー・検索 (必須)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **telescope.nvim** | ファジーファインダー（ファイル検索・grep） | ⭐⭐⭐ 必須 | ❌ 非常に重要 |
| **telescope-fzf-native.nvim** | Telescope高速化 | ⭐⭐ 推奨 | ⚠️ パフォーマンス向上 |
| **plenary.nvim** | Lua関数ライブラリ（依存） | ⭐⭐⭐ 必須 | ❌ 多くのプラグインが依存 |

**推奨**: すべて残す。Telescopeは最も重要なプラグインの1つ。

---

## 📁 ファイルエクスプローラー

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **neo-tree.nvim** | ファイルツリー | ⭐⭐ 推奨 | ✅ Telescopeで代替可 |
| **nui.nvim** | Neo-treeの依存ライブラリ | ⭐⭐ 推奨 | ⚠️ Neo-treeを使うなら必須 |

**推奨**: 
- ファイルツリーをよく使う → 残す
- Telescopeの`:Telescope find_files`で十分 → 削除可

---

## 💻 LSP・補完 (必須)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **nvim-lspconfig** | LSP設定 | ⭐⭐⭐ 必須 | ❌ コード補完・診断に必須 |
| **mason.nvim** | LSPサーバー自動インストール | ⭐⭐⭐ 必須 | ❌ LSP管理に必須 |
| **mason-lspconfig.nvim** | MasonとLSPの統合 | ⭐⭐⭐ 必須 | ❌ LSP管理に必須 |
| **nvim-cmp** | 補完エンジン | ⭐⭐⭐ 必須 | ❌ 自動補完に必須 |
| **cmp-nvim-lsp** | LSP補完ソース | ⭐⭐⭐ 必須 | ❌ LSP補完に必須 |
| **cmp-buffer** | バッファ補完ソース | ⭐⭐ 推奨 | ✅ 削除可 |
| **cmp-path** | パス補完ソース | ⭐⭐ 推奨 | ✅ 削除可 |
| **cmp-cmdline** | コマンドライン補完 | ⭐ オプション | ✅ 削除可 |
| **LuaSnip** | スニペットエンジン | ⭐⭐ 推奨 | ⚠️ 便利だが必須ではない |
| **cmp_luasnip** | LuaSnip補完ソース | ⭐⭐ 推奨 | ⚠️ LuaSnipを使うなら必須 |
| **friendly-snippets** | スニペット集 | ⭐ オプション | ✅ 削除可 |

**推奨**: 
- LSP関連（lspconfig、mason、nvim-cmp、cmp-nvim-lsp）→ すべて残す
- スニペット系（LuaSnip、friendly-snippets）→ 使わないなら削除可
- cmp-buffer、cmp-path → 便利なので残す推奨
- cmp-cmdline → あまり使わないなら削除可

---

## 🌳 Treesitter (推奨)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **nvim-treesitter** | シンタックスハイライト | ⭐⭐⭐ 必須 | ⚠️ 従来のsyntaxに戻る |

**推奨**: 残す。モダンなシンタックスハイライトに必須。

---

## 🔧 Git統合 (推奨)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **gitsigns.nvim** | Git変更表示・hunk操作 | ⭐⭐⭐ 必須 | ⚠️ Git作業に便利 |
| **vim-fugitive** | Gitコマンド統合 | ⭐⭐ 推奨 | ✅ Gitをターミナルで操作するなら不要 |

**推奨**: 
- gitsigns → 残す（変更箇所の可視化が便利）
- vim-fugitive → Git操作をVim内でしたいなら残す、ターミナル派なら削除可

---

## 🛠️ ユーティリティ (一部必須)

| プラグイン | 説明 | 必須度 | 削除可否 |
|-----------|------|--------|---------|
| **Comment.nvim** | コメント操作（gcc） | ⭐⭐⭐ 必須 | ⚠️ 非常に便利 |
| **nvim-autopairs** | 括弧自動閉じ | ⭐⭐ 推奨 | ✅ 好みによる |
| **which-key.nvim** | キーマップヘルプ表示 | ⭐⭐ 推奨 | ✅ 削除可 |
| **trouble.nvim** | 診断リスト表示 | ⭐⭐ 推奨 | ✅ 削除可 |
| **vim-illuminate** | カーソル下の単語ハイライト | ⭐ オプション | ✅ 削除可 |
| **vim-matchup** | %マッチング強化 | ⭐ オプション | ✅ 削除可 |
| **vim-cheatsheet** | チートシート表示 | ⭐ オプション | ✅ 削除可 |

**推奨**:
- Comment.nvim → 残す（gccでコメント化は必須級）
- nvim-autopairs → 好みだが便利
- which-key → キーマップを覚えるまで便利
- trouble → LSP診断を見やすく表示
- vim-illuminate → 邪魔に感じるなら削除
- vim-matchup → 複雑な構文でない限り不要
- vim-cheatsheet → 使ってないなら削除

---

## 📊 削除推奨度の優先順位

### 🔴 削除しても良いプラグイン（影響小）

1. **vim-cheatsheet** - 使ってない可能性が高い
2. **vim-illuminate** - 邪魔に感じる人もいる
3. **vim-matchup** - 基本の%で十分なことが多い
4. **friendly-snippets** - スニペットを使わないなら
5. **cmp-cmdline** - コマンドライン補完をあまり使わない
6. **indent-blankline.nvim** - インデントガイドが不要なら

### 🟡 条件付きで削除可能

1. **neo-tree.nvim + nui.nvim** - Telescopeで十分なら
2. **vim-fugitive** - Git操作をターミナルでするなら
3. **trouble.nvim** - Telescopeの診断機能で十分なら
4. **which-key.nvim** - キーマップを覚えたら
5. **LuaSnip + cmp_luasnip** - スニペットを使わないなら

### 🟢 残すべき（削除非推奨）

- LSP関連すべて
- Telescope関連すべて
- Comment.nvim
- gitsigns.nvim
- nvim-treesitter
- lualine.nvim
- nvim-autopairs

---

## 🗑️ 削除手順

### 個別プラグインを削除する場合

1. 該当するプラグイン設定ファイルから削除
2. Neovimを再起動
3. `:Lazy clean` を実行

### 例: vim-cheatsheetを削除

`nvim/lua/plugins/utilities.lua`を編集：

```lua
-- Vim cheatsheet の部分を削除またはコメントアウト
-- {
--   "reireias/vim-cheatsheet",
--   cmd = "Cheat",
--   keys = {
--     { "<Space>ch", "<cmd>Cheat<cr>", desc = "Cheatsheet" },
--   },
--   config = function()
--     vim.g.cheatsheet_cheat_file = vim.fn.expand("~/dotfiles/vim/cheatsheet/common.md")
--   end,
-- },
```

---

## 💡 推奨構成

### ミニマル構成（約15プラグイン）
- lazy.nvim
- LSP関連（lspconfig, mason, nvim-cmp系）
- telescope + fzf-native
- nvim-treesitter
- Comment.nvim
- gitsigns.nvim
- lualine.nvim
- plenary.nvim
- nvim-web-devicons
- カラースキーム

### バランス構成（現在の設定、約30プラグイン）
- ミニマル構成 + 
- neo-tree
- nvim-autopairs
- which-key
- trouble
- vim-fugitive
- LuaSnip系

### フル構成
- 現在のすべて

---

## 📝 メモ

削除を検討する際の判断基準：
1. **過去1ヶ月で使ったか？** → 使ってないなら削除候補
2. **機能が重複していないか？** → 重複機能は削除候補
3. **動作が重くないか？** → 重い＋使わない = 削除
4. **依存関係は？** → 他のプラグインが依存してないか確認

---

現在の設定は**バランス型**で、一般的な開発に十分な機能を提供しています。
特に不満がなければ、このまま使い続けることを推奨します。
