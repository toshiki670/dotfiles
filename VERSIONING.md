# バージョニングルール

このドキュメントでは、dotfiles プロジェクトのバージョニングルールを定義します。

## セマンティックバージョニング 0.x.x

このプロジェクトは[セマンティックバージョニング 2.0.0](https://semver.org/)に従い、**0.x.x 形式**でバージョン管理を行います。

### バージョン形式

```
v0.MINOR.PATCH
```

- **MAJOR**: 0（開発初期段階・正式リリース前を示す）
- **MINOR**: 破壊的変更を含む機能追加・変更
- **PATCH**: バグ修正や小さな改善

### 0.x.x の意味

[SemVer 仕様](https://semver.org/#spec-item-4)によると：

> Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The public API SHOULD NOT be considered stable.

メジャーバージョン 0 は：

- 初期開発段階を示す
- いつでも何でも変更可能
- パブリック API（設定ファイルの構造など）は安定していないと見なされる
- **破壊的変更を MINOR バージョンで導入できる**

## リリースの自動化

このプロジェクトでは、**Bash スクリプト + GitHub CLI（`gh`）** を使用してリリースプロセスを自動化しています。リリースタイプを手動で選択することで、より柔軟なバージョン管理を実現しています。外部依存を最小限に抑え、シンプルで理解しやすい仕組みを採用しています。

### リリースフロー（GitHub Flow + 手動リリース）

このプロジェクトは **GitHub Flow** に基づいた開発フローを採用し、**手動でリリースを実行**します。

**開発からリリースまでの流れ：**

1. **feature/fix ブランチで開発**
   - `feature/機能名` または `fix/バグ名` ブランチを作成
   - わかりやすいコミットメッセージで記録（Conventional Commits 推奨）
2. **Pull Request を作成**
   - main ブランチへの Pull Request を作成
   - レビュー・テストを実施
3. **main ブランチにマージ**
   - Pull Request をマージ（Squash & Merge 推奨）
   - 複数の PR をまとめてマージ可能
4. **手動でリリースを実行**
   - エンジニアが GitHub Actions から手動でリリースを実行
   - リリースタイプを選択（patch / minor / major）
   - 最新のGitHubリリースまたはGitタグから現在のバージョンを取得
   - バージョン番号を自動計算
   - Git タグを作成（`v`プレフィックス付き）
   - GitHub Release を作成（リリースノート自動生成）

**重要：main ブランチへの直接コミットは禁止**

```bash
# ❌ 直接mainにコミット（禁止）
git checkout main
git commit -m "feat: new feature"
git push origin main

# ✅ Pull Request経由（推奨）
git checkout -b feature/new-feature
git commit -m "feat: add new feature"
git push origin feature/new-feature
# → GitHubでPull Requestを作成してマージ
```

**リリースの実行方法：**

1. GitHub リポジトリの「Actions」タブを開く
2. 「Release」ワークフローを選択
3. 「Run workflow」ボタンをクリック
4. **Release type** を選択：
   - **patch**: バグ修正（0.28.0 → 0.28.1）
   - **minor**: 新機能・破壊的変更（0.28.0 → 0.29.0）
   - **major**: メジャーバージョンアップ（0.x.y → 1.0.0）
5. 「Run workflow」を実行

**バージョンアップのルール：**

手動で選択したリリースタイプに基づいて、最新のGitHubリリースまたはGitタグから現在のバージョンを取得し、自動的に新しいバージョンが計算されます：

- **patch**: バグ修正・小さな改善（0.x.y → 0.x.(y+1)）
- **minor**: 新機能・破壊的変更（0.x.y → 0.(x+1).0）
- **major**: メジャーバージョンアップ（0.x.y → 1.0.0）

**複数の変更をまとめてリリース：**

```bash
# 例：3つのPRをmainにマージ
# PR#1: feat: add zsh completion
# PR#2: fix: correct PATH order
# PR#3: feat: add vim configuration

# → エンジニアが手動でReleaseアクションを実行
# → v0.29.0 として一括リリース（2つのfeatがあるのでMINORアップ）
```

**リリースタイミングの柔軟性：**

- 複数の PR をマージしてから、任意のタイミングでリリース可能
- 緊急のバグ修正は即座にリリース可能
- 機能をまとめて計画的にリリース可能

## バージョンアップの例

### MINOR バージョンアップ（0.x.0 → 0.(x+1).0）

以下のような変更で MINOR バージョンを上げる：

- 設定ファイルの構造変更
- プラグインマネージャーの変更（例：zinit → sheldon）
- エイリアスやコマンドの破壊的変更
- ディレクトリ構造の変更
- 新しいツールへの依存関係追加
- 大きな機能追加や改善

**コミット例：**

```bash
# 新機能の追加
git commit -m "feat: add mise configuration for Node.js management"
# → v0.28.0 → v0.29.0

# 破壊的変更
git commit -m "feat!: migrate from zinit to sheldon

BREAKING CHANGE: Plugin manager changed, reinstall required"
# → v0.28.0 → v0.29.0
```

### PATCH バージョンアップ（0.x.y → 0.x.(y+1)）

以下のような変更で PATCH バージョンを上げる：

- バグ修正
- タイポ修正
- 既存設定の微調整
- ドキュメントの更新（`docs:`を使用）
- 小さな改善や最適化
- コメントの追加

**コミット例：**

```bash
# バグ修正
git commit -m "fix: correct PATH environment variable order"
# → v0.28.0 → v0.28.1

# ドキュメント更新（バージョンアップなし）
git commit -m "docs: update README installation instructions"
# → バージョン変更なし
```

## タグの命名規則

- **形式**: `v0.x.y`（`v` プレフィックス付き）
- **例**: `v0.18.0`, `v0.18.1`, `v0.19.0`
- ❌ 使用しない: `0.18.0`（プレフィックスなし）, `version-0.18.0`

## 将来の 1.0.0 リリース

このプロジェクトが以下の条件を満たした時、1.0.0 をリリースすることを検討します：

- 設定ファイルの構造が安定している
- ドキュメントが十分に整備されている
- インストールプロセスが確立している
- 複数のプラットフォームで安定動作している
- コミュニティからのフィードバックが安定している

**1.0.0 以降のバージョニング：**

- MAJOR（x.0.0）: 破壊的変更
- MINOR（0.x.0）: 後方互換性のある機能追加
- PATCH（0.0.x）: 後方互換性のあるバグ修正

## 開発者向け情報

### リリースノートについて

リリースノートは **GitHub Releases の自動生成機能**（`--generate-notes`）を使用します。

- GitHubが前回のタグからのコミットログを自動的に収集
- PRやコミットメッセージがリリースノートとして表示される
- CHANGELOG ファイルは管理しない（GitHub Releases 参照）

### ローカルでの確認

```bash
# 現在のバージョンを確認
gh release view --json tagName -q '.tagName'
# または
git describe --tags --abbrev=0

# バージョンバンプスクリプトのヘルプを表示
./bin/bump_version -h

# 次のバージョンを確認（純粋関数として動作）
./bin/bump_version v0.28.0 patch   # Output: v0.28.1
./bin/bump_version v0.28.0 minor   # Output: v0.29.0
./bin/bump_version v0.28.0 major   # Output: v1.0.0
```

### 必要なツール

- **git**: バージョン管理システム
- **gh**: GitHub CLI（バージョン取得とリリース作成に使用、CI/CDで使用）

**注意**: `bin/bump_version` は純粋関数として設計されており、現在のバージョンとバンプタイプを受け取って新しいバージョンを返すだけです。実際のバージョン取得、タグ作成、GitHubリリース作成はCI/CDワークフローで実行されます。

詳細は [`CONTRIBUTING.md`](CONTRIBUTING.md) を参照してください。

## 参考リンク

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Semantic Versioning 2.0.0（日本語）](https://semver.org/lang/ja/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Conventional Commits（日本語）](https://www.conventionalcommits.org/ja/)
- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
