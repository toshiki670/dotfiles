# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## リポジトリ構造

chezmoi で管理。`home/` がソースで `chezmoi apply` でホームディレクトリへデプロイされる（`home/dot_config/` → `~/.config/`）。`home/.chezmoiscripts/` に apply 前後のフック（パッケージアップグレード、シンボリックリンク作成、ヘルスチェック、cargo install）がある。

CLI コマンドはリポジトリルートの **Cargo workspace**。配布物は root package `dotfiles` 1 つに統合され、各コマンドはその複数 bin（`src/bin/<name>.rs` の数行シム）として並ぶ。ロジックは系ごとの module（`src/core/` 本体、`src/clip/` `src/gcm/` `src/gh_clone/` `src/git_upstream/`、fzf 系ピッカー `src/fzf_picker/`（worktree パーサ等の共有 lib + cdabbr / fzf-gh / fzf-ghq-cd / fzf-worktree-remove の各 bin）、環境メンテナンス `src/env_tools/`（banner 等の共有 lib + cleanup-env / upgrade-env）、バックグラウンド worker `src/workers/`（daily-check / git-background-fetch））。version は単一・SoT はタグ `v{version}`。`chezmoi apply` のフックが root package を一度の `cargo install` で `~/.cargo/bin` へ配布する（`cargo install --git <repo>` 一発でも入る）。開発・保守ツールは `tools/` 配下で非配布・版なし: lint/format オーケストレータ `dotfiles-lint`（`mise run lint` / `check` → `cargo run -p dotfiles-lint`、mise 供給の shfmt / shellcheck / taplo / stylua / rumdl / ruff / chezmoi を呼ぶ）、Neovim プラグイン同期 `v-sync`（`mise run v-sync`）。リリースは release-plz（単一版の git タグ + GitHub Release、crates.io へは publish しない）。

- セットアップ・ツール一覧・Rust コマンド → [README.md](README.md)
- lint/check・テスト・リリース手順 → [CONTRIBUTING.md](CONTRIBUTING.md)
- バージョニングルール → [CONTRIBUTING.md](CONTRIBUTING.md#リリースプロセス)
- カラーテーマ設定の一覧・変更手順 → [COLOR.md](COLOR.md)
