# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## リポジトリ構造

chezmoi で管理。`home/` がソースで `chezmoi apply` でホームディレクトリへデプロイされる（`home/dot_config/` → `~/.config/`）。`home/.chezmoiscripts/` に apply 前後のフック（パッケージアップグレード、シンボリックリンク作成、ヘルスチェック、cargo install）がある。

CLI コマンドはリポジトリルートの **Cargo workspace**。root package = `dotfiles` 本体（core、`src/main.rs`、version SoT・タグ `v{version}`）。各コマンドは `crates/<name>/`（color / copy-obj / v-sync / gh-clone / gcm / git-upstream）、バックグラウンド worker は `crates/dotfiles-workers/`（2 bin）、共有 lib は `crates/dotfiles-support/`。各配布物・lib は独立版（release-plz の per-package タグ `<crate>-v<ver>`）。`chezmoi apply` のフックが配布クレートのみを `cargo install` で `~/.cargo/bin` へ配布する（support lib と lint は非 install）。lint/format は Rust 製オーケストレータ `dotfiles-lint`（`tools/dotfiles-lint/`、版なし・`mise run lint` / `check` → `cargo run -p dotfiles-lint`）が mise 供給のツール（shfmt / shellcheck / taplo / stylua / rumdl / ruff / chezmoi）を呼ぶ。リリースは release-plz（git タグ + GitHub Release、crates.io へは publish しない）。

- セットアップ・ツール一覧・Rust コマンド → [README.md](README.md)
- lint/check・テスト・リリース手順 → [CONTRIBUTING.md](CONTRIBUTING.md)
- バージョニングルール → [CONTRIBUTING.md](CONTRIBUTING.md#リリースプロセス)
- カラーテーマ設定の一覧・変更手順 → [COLOR.md](COLOR.md)
