# Fish 設定の Zsh からの段階的移行リスト

現在の Zsh 設定（`home/dot_config/zsh/`）を基準に、Fish へ移行できる項目を段階別にまとめたものです。

---

## 現状

| 項目 | Zsh | Fish |
|------|-----|------|
| エントリ | `dot_zshrc.tmpl`（複数 config を include） | `config.fish`（ほぼ空） |
| プロンプト | Pure + ci-status（非同期） | `fish_prompt.fish`（nim 風・既に充実） |
| プラグイン | Sheldon（zeno, fzf, 補完など） | 未導入 |

---

## Phase 1: 環境変数・PATH・エイリアス（そのまま移行しやすい）

| Zsh の元 | 内容 | Fish での対応 |
|----------|------|----------------|
| `dot_zshrc.tmpl` | `DOTFILES` | `set -gx DOTFILES (chezmoi source-path)` など |
| `common.zsh` | `PATH`: DOTFILES/bin, ~/.local/bin, ~/.cargo/bin | `fish_user_paths` または `set -gx PATH ...` |
| `common.zsh` | `reload` | `alias reload='exec fish -l'` |
| `common.zsh` | `ipecho`, `ps-grep`, `df`, `cp`/`mv` (Linux) | `alias` / `function` で同様に定義 |
| `common.zsh` | 拡張子別実行 (txt→vim, html→open 等) | Fish では `open` の関数や `bind` で代替検討（後回し可） |
| `docker.zsh` | `dce='docker-compose exec'` | `alias dce='docker-compose exec'` |
| `git.zsh` | `g='git'`, git-flow 系エイリアス | `abbr` または `alias` |
| `vim.zsh` | `vim=nvim`, `v`, `vim-utf8` 等, `vimrc` | `alias` / `function` |
| `ruby.zsh` | `be`, `kill-rails`, `check-rails` | `alias` / `function` |
| `python.zsh` | 拡張子 py→python | 同上（必要なら後で） |
| `dotfiles.zsh` | `dotfiles='chezmoi'`, `dotfiles-latest` | `alias` + 関数を Fish 構文で書き直し |
| `eza.zsh` | `ls`/`la`/`lt` の eza ラップ | `function` で同様に（`lt` は引数パースを Fish で実装） |
| `yt-dlp.zsh` | `yt`, `yt-chat`（YT_BROWSER 使用） | `alias`（環境変数はそのまま） |
| `pbcopy.zsh` | `pbcopy-path`, `pbcopy-file` | `function` で書き直し（macOS のみ） |
| `brew.zsh` | Homebrew PATH を先頭に | `config.fish` で `fish_user_paths` に `/opt/homebrew/bin` を先頭で追加 |

**進め方**: `config.fish` と `conf.d/` に「環境変数」「alias」「function」を少しずつ追加。chezmoi の `DOTFILES` はテンプレートで注入する想定。

---

## Phase 2: ツールの Fish 対応（init やプラグイン）

| Zsh の元 | 内容 | Fish での対応 |
|----------|------|----------------|
| `zoxide.zsh.tmpl` | `zoxide init zsh`, `bz='z ..'`, ^j^f で zi | `zoxide init fish`。`abbr bz 'z ..'`。^j^f は `bind` で fzf + zoxide の関数を割り当て |
| `fzf.zsh` | FZF_DEFAULT_OPTS/COMMAND, ^j^h, ^j^t, ^j^g (git branch) | 環境変数はそのまま。キーバインドは `fzf.fish` プラグインまたは自前 `bind` + 関数 |
| `mise.zsh.tmpl` | `mise activate zsh` | `mise activate fish` を config.fish で実行 |
| `history.zsh` | HISTSIZE, share_history, 履歴検索 | Fish は `history` コマンドと `$fish_history`。共有・件数は `fish_history` の設定で調整 |
| `cd.zsh` | `setopt auto_pushd` | `cd` のラッパーで `dirs` 相当を実装するか、`prevd`/`nextd` は標準であるので必要に応じて |

**進め方**:  
- mise / zoxide は `config.fish` に 1 行ずつ追加。  
- fzf は [fzf.fish](https://github.com/PatrickF1/fzf.fish) を導入し、Zsh の ^j^h / ^j^t / ^j^g に近いキーをカスタム設定するか、後から bind で合わせる。

---

## Phase 3: 補完・キーバインド（Fish の仕組みに合わせる）

| Zsh の元 | 内容 | Fish での対応 |
|----------|------|----------------|
| `docker.zsh` | docker / docker-compose の補完を curl で取得 | Fish は `completions/` に `.fish` を置くか、`docker completion fish` 等で取得 |
| `completion.zsh` | compinit, zstyle, menu select | Fish 標準の補完 + 必要ならプラグイン。大文字小文字無視などは別途設定 |
| `fzf-tab.zsh` | タブ補完を fzf で選択 | fzf.fish のタブ連携や、自前で `bind \t` と fzf を組み合わせる |
| `clipboard-completion.zsh` | クリップボード 1 行を補完候補に | Fish ではカスタム補完または `complete -c` で候補を追加する関数を検討（後回し可） |
| `fzf.zsh` | ^j^h history, ^j^t file, ^j^g branch | fzf.fish のキーを ^j 系に変更するか、自前 bind |

**進め方**: まずは標準の補完だけで運用し、fzf 連携は fzf.fish 導入後にキーだけ合わせる。clipboard 補完は必要性が高ければ Phase 4 で。

---

## Phase 4: プロンプト・非同期・Zsh 固有

| Zsh の元 | 内容 | Fish での対応 |
|----------|------|----------------|
| `ci-status.zsh` | Pure プロンプトに GitHub Actions の PR/checks を非同期表示 | Fish では `fish_prompt` 内で `gh pr view` / `gh pr checks` を呼ぶ場合、非同期は `fish --private` やバックグラウンドジョブで再実装が必要。既存の fish_prompt は Git まで表示しているので、CI 表示は「同じ見た目」を目指すなら関数を分離して実装 |
| `daily-check.zsh` | 起動時に 1 日 1 回 brew/mise outdated を表示 | 同様のロジックを Fish で関数化（日付ファイルでキャッシュ、バックグラウンド実行） |
| Sheldon / zeno | プラグイン・スニペット | Fish には Sheldon はない。fzf.fish で fzf 部分を賄い、zeno 相当の機能は必要なら別プラグインや自前関数で |

**進め方**:  
- daily-check は Zsh のロジックを Fish の `function` に移植すればよい。  
- ci-status は「PR があるときだけプロンプトに表示」する仕様を Fish で再実装するか、一旦なしで進めて後から追加するか選ぶ。

### Fish の ci-status（簡略版）

Zsh の ci-status に完全に合わせることはしていない。

- **更新中**  
  バックグラウンドで `gh` を叩いている間は、プロンプトには「更新中」アイコン（◐）だけを表示する。親プロセスへのフィードバック（SIGUSR1 で repaint など）は行わない。
- **結果の保存**  
  キャッシュファイルは使わず、universal 変数 `_ci_cache`（エントリ形式 `key|value|ts`）に保存する。`key` にディレクトリとブランチ名を含めるので、ディレクトリ移動後も別ブランチの情報が混ざらない。
- **表示**  
  キャッシュが有効な間はその値を表示。キャッシュがない／古いときは取得をバックグラウンドで開始し、そのあいだは ◐ のみ表示。次にプロンプトが描画されるとき（次のコマンド後）に結果が表示される。

---

## Phase 5: 条件付き・OS 別（そのまま移行）

| Zsh の元 | 内容 | Fish での対応 |
|----------|------|----------------|
| `pacman.zsh` | `lookPath "pacman"` で swap-pacman-mirrorlist | `if command -q pacman; ... end` で関数定義 |
| `yt-dlp.zsh` | `env "YT_BROWSER"` があるときだけ | `if set -q YT_BROWSER; ... end` |
| `c.zsh` | rungcc, 拡張子 c/cpp→rungcc | Fish で `function` 化（md5sum は macOS では `md5` など要確認） |

---

## 推奨する移行順序（まとめ）

1. **Phase 1**  
   - DOTFILES, PATH, reload, ipecho, ps-grep, df, dce, g, vim/v, dotfiles/dotfiles-latest, eza の ls/la/lt, pbcopy-path/file, brew PATH, be/kill-rails/check-rails, yt/yt-chat など、エイリアス・関数のうち使っているものから Fish に追加する。

2. **Phase 2**  
   - mise activate, zoxide init, fzf 環境変数と ^j^f（zoxide interactive）を追加。  
   - 履歴は Fish デフォルトのままか、必要なら `fish_history` を調整。

3. **Phase 3**  
   - docker/docker-compose の補完を Fish 用に取得。  
   - fzf.fish を入れ、^j^h / ^j^t / ^j^g に近い操作をキーバインドで合わせる。

4. **Phase 4**  
   - daily-check を Fish で実装。  
   - CI 表示が必要なら、`fish_prompt` 用の ci-status 相当を別関数として実装。

5. **Phase 5**  
   - pacman / yt-dlp / c は必要に応じて条件付きで追加。

---

## ファイル配置の目安（Fish）

- **環境変数・PATH・mise/zoxide**: `config.fish` または `conf.d/01-env.fish`
- **エイリアス・abbr**: `config.fish` または `conf.d/02-aliases.fish`
- **関数（dotfiles-latest, daily-check, pbcopy-path 等）**: `functions/` に 1 関数 1 ファイル、または `conf.d/` にまとめる
- **fzf キーバインド**: fzf.fish の設定か `conf.d/03-fzf.fish`
- **プロンプト**: `functions/fish_prompt.fish`（path / branch / dirty *（ピンク）/ ⇡⇣（Pure 風）/ ci-status）

この順で段階的に移すと、Zsh に近い体験を Fish でも再現しやすいです。
