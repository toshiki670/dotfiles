# COLOR.md — カラーテーマ インデックス

この dotfiles のカラーテーマ設定を一覧化したもの。**テーマを変更するときはここを起点**にすると、関係ファイルを素早く特定できる。

## 現在の構成

- **light = One Half Light / dark = Ayu**
- macOS のシステム外観（ライト/ダーク）に**自動追従**する。

起点は **Ghostty**。OS の外観に応じて背景色と ANSI パレットを切り替える（One Half Light = `#fafafa` / Ayu = `#0b0e14`）。他のツールはその**端末背景（OSC 11）**を検出して追従する。パス表記は chezmoi ソース（`home/`）基準。デプロイ先は `~/` 配下。

## インデックス（ツール別）

| ツール | 設定ファイル（`home/` 基準） | light テーマ | dark テーマ | 追従方式 | 利用可能テーマの確認 |
| --- | --- | --- | --- | --- | --- |
| **Ghostty** | `dot_config/ghostty/config`（`theme =` 行） | One Half Light | Ayu | Ghostty ネイティブ。`theme = light:…,dark:…` で OS 外観に追従 | `ghostty +list-themes` |
| **Fish**（構文色） | `dot_config/fish/conf.d/90-fish-theme.fish` + `dot_config/fish/themes/OneHalfLight-Ayu.theme` | `[light]` セクション | `[dark]` セクション | `fish_terminal_color_theme`（OSC 11）→ `fish_config theme choose` が変数変化に追従 | `fish_config theme list` |
| **Neovim** | `dot_config/nvim/lua/plugins/colorscheme.lua` | `onehalflight`（`sonph/onehalf`） | `ayu-dark`（`Shatur/neovim-ayu`） | `&background`（**起動時**に OSC 11 検出）→ `OptionSet` で colorscheme 切替 | `:colorscheme <Tab>` |
| **eza** | `dot_config/fish/conf.d/40-eza.fish`（`EZA_COLORS`） | グレー明度（濃いめ） | グレー明度（薄いめ） | 色付き要素は Ghostty パレット追従（ANSI）。グレーのメタデータのみ `fish_terminal_color_theme` で明度切替 | `man eza_colors` |
| **fzf** | `dot_config/fish/conf.d/05-fzf-env.fish`（`FZF_DEFAULT_OPTS` の `--color`） | ANSI 追従 | ANSI 追従 | `--color` を ANSI 番号（`-1`=端末デフォルト / `0`〜`15`）で指定。Ghostty パレット追従でシェル配線不要 | `man fzf` の COLOR セクション |
| **git-delta** | `dot_config/git/configs/delta`（`[delta "theme-light/dark"]`）+ `dot_config/fish/conf.d/50-delta.fish`（`DELTA_FEATURES`） | `OneHalfLight` | `ayu-dark` | `DELTA_FEATURES` を `fish_terminal_color_theme` に追従。git 実行ごとに反映 | `delta --list-syntax-themes` |
| **bat** | `dot_config/bat/config`（`--theme-light/dark`） | `OneHalfLight` | `ayu-dark` | bat ネイティブ `--theme="auto"`（bat 自身が OSC 11 検出）。シェル配線不要 | `bat --list-themes` |

### カスタムテーマ（標準に無いもの）

- `dot_config/bat/themes/ayu-dark.tmTheme` — **Ayu** の bat/delta 用テーマ。delta(dark) と bat(dark) が**共有**する。`dempfi/ayu` 公式配色から生成。
- `.chezmoiscripts/run_onchange_after_build-bat-cache.sh.tmpl` — 上記テーマが変わると `bat cache --build` を自動実行（chezmoi apply 時）。これが無いと delta/bat が `ayu-dark` を解決できない。

## 追従の仕組み（要点）

- **Ghostty が起点**：OS 外観 → 端末の背景色 + ANSI 16 色パレットを切替。
- **Fish / eza / delta** は `fish_terminal_color_theme`（fish が最初の対話プロンプト以降、OSC 11 で背景を検出して `light`/`dark`/`unknown` を設定する read-only 変数）を共有して追従する。
- **fzf** は `--color` を ANSI 番号（`-1`/`0`〜`15`）で指定するだけで Ghostty のパレット切替に追従する。シェル変数にもネイティブ検出にも依存しない（eza の色付き要素と同じ ANSI 追従）。
- **nvim** は標準機能で **起動時のみ** `&background` を検出（起動中の OS 切替は次回起動で反映）。
- **bat** は `--theme=auto` で自前検出するため、シェル変数に依存しない。

## テーマを変更するには

light / dark を別のテーマセットに変える手順。**上から順に**編集すれば一通り揃う。

1. **Ghostty** `dot_config/ghostty/config`
   `theme = light:<新Light>,dark:<新Dark>` を書き換える（テーマ名は `ghostty +list-themes`）。
2. **Fish** `dot_config/fish/themes/OneHalfLight-Ayu.theme`
   `[light]` / `[dark]` の各 `fish_color_*` を新テーマの配色に書き換える（color-theme-aware なので両方）。別の標準テーマに丸ごと変えるなら、`conf.d/90-fish-theme.fish` の `fish_config theme choose <名前>` を変更（`fish_config theme list` で確認）。
3. **Neovim** `dot_config/nvim/lua/plugins/colorscheme.lua`
   プラグイン（`Shatur/neovim-ayu` / `sonph/onehalf`）と、`apply()` 内の `onehalflight` / `ayu-dark` を新 colorscheme 名に変更。プラグインを変えたら `nvim --headless "+Lazy! sync" +qa` と `lazy-lock.json` の更新を忘れずに。
4. **eza**（任意）`dot_config/fish/conf.d/40-eza.fish`
   グレー系メタデータの明度だけ。色付き要素は Ghostty 追従なので通常は触らなくてよい。
5. **fzf**（任意）`dot_config/fish/conf.d/05-fzf-env.fish`
   `--color` は ANSI 番号で Ghostty 追従するため通常は触らなくてよい。アクセント色の色相を変えたい場合のみ `--color` の ANSI 番号（`-1`/`0`〜`15`）を調整（`man fzf` の COLOR セクション）。
6. **git-delta** `dot_config/git/configs/delta`
   `[delta "theme-light"]` / `[delta "theme-dark"]` の `syntax-theme` を変更（名前は `delta --list-syntax-themes`）。
7. **bat** `dot_config/bat/config`
   `--theme-light` / `--theme-dark` を変更（名前は `bat --list-themes`）。

> **標準に無いテーマ**（今回の Ayu のような）を使う場合:
>
> 1. `<テーマ>.tmTheme` を `dot_config/bat/themes/` に置く（`.sublime-color-scheme` は bat が読めないことがあるので `.tmTheme` 推奨）。
> 2. `bat cache --build` を実行（chezmoi 経由なら hook が自動実行）。テーマ名は**ファイル名**（拡張子なし）になる。
> 3. delta / bat はこの名前を `syntax-theme` / `--theme-dark` で参照できる。delta と bat はこのテーマ名前空間を共有する。

## 適用と確認

- **適用**: `chezmoi apply`（bat テーマを変えた場合は hook が `bat cache --build` を実行）。
- **確認**:
  - bat: `bat <ファイル>`（`--theme=auto` なので即時反映）
  - delta: **新しい fish** で `git diff` / `git show`
  - nvim: 再起動
  - Fish / eza: **新しい fish セッション**
  - Ghostty: 新しいウィンドウ / タブ
- いずれも端末背景（または OS 外観）に依存するため、macOS の外観を切り替えて確認する。
