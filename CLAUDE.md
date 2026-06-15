# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## リポジトリ構造

chezmoi で管理。`home/` がソースで `chezmoi apply` でホームディレクトリへデプロイされる（`home/dot_config/` → `~/.config/`）。`home/.chezmoiscripts/` に apply 前後のフック（パッケージアップグレード、シンボリックリンク作成、ヘルスチェック、cargo install）がある。

CLI コマンドの一部はリポジトリルートの **Rust クレート**（`Cargo.toml` / `src/bin/<name>/` / 共有は `src/lib.rs`、version SoT）。`chezmoi apply` のフックが `cargo install` で `~/.cargo/bin` へ配布する。lint/format は Rust 製オーケストレータ `dotfiles-lint`（`mise run lint` / `check`）が mise 供給のツール（shfmt / shellcheck / taplo / stylua / rumdl / ruff / chezmoi）を呼ぶ。リリースは release-plz（git タグ + GitHub Release、crates.io へは publish しない）。

- セットアップ・ツール一覧・Rust コマンド → [README.md](README.md)
- lint/check・テスト・リリース手順 → [CONTRIBUTING.md](CONTRIBUTING.md)
- バージョニングルール → [CONTRIBUTING.md](CONTRIBUTING.md#リリースプロセス)
- カラーテーマ設定の一覧・変更手順 → [COLOR.md](COLOR.md)
