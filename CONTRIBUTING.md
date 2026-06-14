# コントリビューションガイド

## 開発環境のセットアップ

### 必要なツール

```bash
# Homebrew経由でインストール
brew install git gh mise
```

```bash
# Nix（lint/check に必要）
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

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

このリポジトリは [python-semantic-release](https://python-semantic-release.readthedocs.io/) を使用して、Conventional Commits に基づくバージョン管理を自動化しています。バージョンの source of truth は `pyproject.toml` の `[project] version` フィールドです。

### バージョン決定ルール

| コミットタイプ | バンプ | 例 |
| --- | --- | --- |
| `feat!:` / `BREAKING CHANGE:` フッター | **major** | `v0.54.2` → `v1.0.0` |
| `feat:` | **minor** | `v0.54.2` → `v0.55.0` |
| `fix:` / `perf:` | **patch** | `v0.54.2` → `v0.54.3` |
| `chore:` / `docs:` / その他 | なし | バージョン変更なし |

### リリースフロー

1. `feat:` / `fix:` / `feat!:` などのコミットが `main` に push されると、semantic-release が自動でバージョンを決定する
2. `pyproject.toml` のバージョンと `CHANGELOG.md` を更新するコミットを直接 `main` に push する
3. Git タグと GitHub Release が自動作成される

手動操作は不要。`main` へのプッシュが自動リリースのトリガー。

### 現在のバージョン確認

```bash
# Git タグから確認
git describe --tags --abbrev=0

# pyproject.toml から確認
grep '^version' pyproject.toml
```

## テスト

変更をコミットする前に、以下を確認してください：

### Nix lint/check（推奨）

```bash
# 自動修正 + チェック
mise run lint

# チェックのみ
mise run check
```

`nix/lint`（および `tests/lint`）を変更したときのみ、以下で lint ランナー実装の品質確認を行ってください:

```bash
nix run .#lint-tests
nix run .#lint-typecheck
nix run .#lint-stylecheck
```

補足:

- CI は `nix flake check -L` と `nix run .#lint-tests` を実行します。
- 詳細ログが必要な場合は `mise run check -- --summary --json` を使ってください。

## コードスタイル

`mise run lint` で適用される linter / formatter に従う。

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
