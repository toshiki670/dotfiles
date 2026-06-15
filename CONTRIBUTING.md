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

このリポジトリは [release-plz](https://release-plz.dev/) を使用して、Conventional Commits に基づくバージョン管理を自動化しています。version の source of truth は各 `Cargo.toml`、CHANGELOG は git-cliff（`cliff.toml`）で生成します。crates.io へは publish せず、git タグ + GitHub Release のみを作成します。

**Cargo workspace の per-package バージョニング**: 配布物（root `dotfiles` + 各コマンド + workers）と共有 lib（`dotfiles-support`）はそれぞれ独立に版を振ります。タグは root `dotfiles` が `v{version}`（従来どおり）、その他の member が `<crate>-v{version}`。CHANGELOG は各クレート直下に per-package で生成されます。開発専用ツール `tools/dotfiles-lint` は `release = false` で release-plz の対象外（版を振りません）。下表のバンプ規則はパッケージごとに、そのパッケージに触れたコミットの type で判定されます。

### バージョン決定ルール

| コミットタイプ | バンプ | 例 |
| --- | --- | --- |
| `feat!:` / `BREAKING CHANGE:` フッター | **major** | `v0.68.0` → `v1.0.0` |
| `feat:` | **minor** | `v0.68.0` → `v0.69.0` |
| `fix:` / `perf:` | **patch** | `v0.68.0` → `v0.68.1` |
| `chore:` / `docs:` / その他 | なし | バージョン変更なし |

### リリースフロー

1. `mise run release-prepare`（または GitHub の Release Prepare workflow を `workflow_dispatch`）で release-plz が Release PR（`release-*` ブランチ）を作成・更新する。
2. Release PR には `Cargo.toml` の version と `CHANGELOG.md` の更新が含まれる（必要なら PR 上で version を手動調整できる）。
3. Release PR を `main` にマージすると Release Publish workflow が走り、per-package タグ（root は `v{version}`、その他は `<crate>-v{version}`）と GitHub Release が作成される。

`main` への直接 push は不要・不可。Release PR のマージがリリースのトリガー（ブランチ保護と両立）。

### 現在のバージョン確認

```bash
# root dotfiles（core）
git describe --tags --abbrev=0 --match 'v[0-9]*'
# 各コマンド（例: color）
git describe --tags --abbrev=0 --match 'color-v*'
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

lint は Rust 製オーケストレータ `dotfiles-lint`（`tools/dotfiles-lint`）が、mise 供給の
各ツールを呼んで実行します。コマンドや lint オーケストレータを変更したときは、以下で
workspace 全体のテスト（各クレートの unit + E2E）を確認してください:

```bash
cargo test --workspace
```

補足:

- CI は `cargo run -p dotfiles-lint -- check`（lint）と `cargo test --workspace`（unit + E2E）を
  実行します。
- 詳細ログが必要な場合は `cargo run -p dotfiles-lint -- check --summary --json` を使ってください。

## コードスタイル

`mise run lint` で適用される linter / formatter に従う。

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
