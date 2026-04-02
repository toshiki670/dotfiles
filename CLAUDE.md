# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## リポジトリ構造

chezmoi で管理。`home/` がソースで `chezmoi apply` でホームディレクトリへデプロイされる（`home/dot_config/` → `~/.config/`）。`home/.chezmoiscripts/` に apply 前後のフック（パッケージアップグレード、シンボリックリンク作成、ヘルスチェック）がある。

- セットアップ・ツール一覧 → [README.md](README.md)
- lint/check・テスト・リリース手順 → [CONTRIBUTING.md](CONTRIBUTING.md)
- バージョニングルール → [VERSIONING.md](VERSIONING.md)
