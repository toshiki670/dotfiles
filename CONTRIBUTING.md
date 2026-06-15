# コントリビューションガイド

## 開発環境のセットアップ

### 必要なツール

```bash
# Homebrew経由でインストール
brew install git gh mise fish
```

lint/format ツール群（shfmt / shellcheck / taplo / stylua / rumdl / ruff / chezmoi）と
Rust ツールチェーンは、リポジトリの `mise.toml` から `mise install` で供給されます
（`fish` のみ brew）。

dotfiles の利用に必要なツールは [`README.md`](README.md) を参照してください。

## コミット規約

[Conventional Commits](https://www.conventionalcommits.org/ja/) 形式を推奨（任意）。

### このリポジトリ固有の判断基準

- `.claude/` 配下のスキルや `home/.chezmoiscripts/` 等、dotfiles 内部でのみ使われるツール・スクリプトの追加・変更は `chore:` を使う（`feat:` ではない）。
  - `feat:` はホームディレクトリに展開され、ユーザー環境に影響する設定・機能の追加に使う。

## ブランチ戦略

**GitHub Flow** を採用。ブランチ命名: `<type>/[<id>-]<short-description>`

- `<id>` は Issue・チケット番号（例: `42`、`PROJECT-8`）。存在する場合のみ付与し、`-` で繋ぐ。
- 例: `feat/42-add-login`、`fix/PROJECT-8-null-check`、`chore/update-deps`

## リリースプロセス

このリポジトリは [release-please](https://github.com/googleapis/release-please) を使用して、Conventional Commits に基づくバージョン管理を自動化しています。

### バージョン決定ルール

| コミットタイプ | バンプ | 例 |
| --- | --- | --- |
| `feat!:` / `BREAKING CHANGE:` フッター | **major** | `v0.54.2` → `v1.0.0` |
| `feat:` | **minor** | `v0.54.2` → `v0.55.0` |
| `fix:` | **patch** | `v0.54.2` → `v0.54.3` |
| `chore:` / `docs:` / その他 | なし | バージョン変更なし |

### リリースフロー

1. `feat:` / `fix:` / `feat!:` などのコミットが `main` に push されると、release-please が Release PR を自動作成・更新する
2. Release PR には `CHANGELOG.md` と `.release-please-manifest.json` の更新が含まれる
3. Release PR をマージすると GitHub Release とタグが自動作成される

手動操作は不要。Release PR のマージがリリースのトリガー。

### 現在のバージョン確認

```bash
git describe --tags --abbrev=0
```

## テスト

変更をコミットする前に、以下を確認してください：

### lint/check

```bash
# 自動修正 + チェック
mise run lint

# チェックのみ
mise run check
```

lint は Rust 製オーケストレータ `dotfiles-lint`（`src/bin/dotfiles-lint`）が、mise 供給の
各ツールを呼んで実行します。オーケストレータ自体を変更したときは、以下でロジックの
ユニットテストを確認してください:

```bash
cargo test
```

補足:

- CI は `cargo run --bin dotfiles-lint -- check`（lint）と `cargo test`（オーケストレータの
  テスト）を実行します。
- 詳細ログが必要な場合は `cargo run --bin dotfiles-lint -- check --summary --json` を使ってください。

## コードスタイル

`mise run lint` で適用される linter / formatter に従う。

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
