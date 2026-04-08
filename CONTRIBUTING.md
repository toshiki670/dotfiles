# コントリビューションガイド

## 開発環境のセットアップ

### 必要なツール

```bash
# Homebrew経由でインストール
brew install git gh zsh mise
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

### リリースの実行方法

Claude Code の `/release` スキルを使用する（推奨）：

```text
/release
```

未リリースの PR を自動分析し、バージョンバンプの種別（`minor` / `patch`）を提案した上で確認後にワークフローをトリガーする。

#### 手動で実行する場合

```bash
# パッチリリース (0.28.0 → 0.28.1)
mise run release-patch

# マイナーリリース (0.28.0 → 0.29.0)
mise run release-minor
```

バージョンバンプの種別（`minor` / `patch`）の判断基準は [`VERSIONING.md`](VERSIONING.md) を参照。

### ローカルでのバージョン確認

```bash
# 現在のバージョンを確認
gh release view --json tagName -q '.tagName'
# または
git describe --tags --abbrev=0

# バージョンバンプスクリプトのヘルプを表示
./bin/bump_version -h

# 次のバージョンを確認
./bin/bump_version v0.28.0 patch   # Output: v0.28.1
./bin/bump_version v0.28.0 minor   # Output: v0.29.0
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

### ci-status の E2E テスト（該当する場合）

`ci-status.zsh` を変更した場合は、E2Eテストを実行してください：

```bash
# 全てのuse-caseを並列実行
zsh tests/ci-status/run-all.zsh
```

テストは `tests/ci-status/` 配下に配置され、各use-caseは独立したzshスクリプトとして並列実行されます。詳細は各use-caseファイルを参照してください。

## コードスタイル

`mise run lint` で適用される linter / formatter に従う。

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
