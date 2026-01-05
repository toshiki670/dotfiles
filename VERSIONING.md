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

このプロジェクトでは、**semantic-release** と **Conventional Commits** を使用してリリースプロセスを完全自動化しています。

### Conventional Commits

コミットメッセージは [Conventional Commits](https://www.conventionalcommits.org/) 形式に従います：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**主要なコミットタイプ：**

- `feat`: 新機能（MINOR バージョンアップ）
- `fix`: バグ修正（PATCH バージョンアップ）
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更（空白、フォーマットなど）
- `refactor`: リファクタリング
- `perf`: パフォーマンス改善
- `test`: テストの追加・修正
- `chore`: ビルドプロセスやツールの変更
- `ci`: CI 設定ファイルの変更

**破壊的変更の表記：**

```
feat!: プラグインマネージャーをsheldonに変更

BREAKING CHANGE: zinitからsheldonに移行したため、再インストールが必要です
```

破壊的変更（`BREAKING CHANGE`）は 0.x.x では MINOR バージョンをアップします。

### リリースフロー（GitHub Flow + 手動リリース）

このプロジェクトは **GitHub Flow** に基づいた開発フローを採用し、**手動でリリースを実行**します。

**開発からリリースまでの流れ：**

1. **feature/fix ブランチで開発**
   - `feature/機能名` または `fix/バグ名` ブランチを作成
   - Conventional Commits 形式でコミット
2. **Pull Request を作成**
   - main ブランチへの Pull Request を作成
   - レビュー・テストを実施
3. **main ブランチにマージ**
   - Pull Request をマージ（Squash & Merge 推奨）
   - 複数の PR をまとめてマージ可能
4. **手動でリリースを実行**
   - エンジニアが GitHub Actions から手動でリリースを実行
   - semantic-release がコミットメッセージを解析
   - バージョン番号を自動決定
   - Git タグを作成（`v`プレフィックス付き）
   - GitHub Release を作成（リリースノート自動生成）
   - package.json を更新してコミット

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
4. オプション：
   - **Dry run**: チェックを入れると実際のリリースは行わず、確認のみ実行
5. 「Run workflow」を実行

**バージョンアップのルール（自動判定）：**

前回のリリースから現在の main ブランチまでのコミットを解析してバージョンを決定：

- `feat:` コミットが 1 つでも含まれる → MINOR バージョンアップ（0.x.0 → 0.(x+1).0）
- `fix:` コミットのみ → PATCH バージョンアップ（0.x.y → 0.x.(y+1)）
- `BREAKING CHANGE:` が含まれる → MINOR バージョンアップ（0.x.x 系では）
- `docs:`, `chore:` など → バージョンアップなし

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

リリースノートは **GitHub Releases の自動生成機能** を使用します。

- semantic-release が Conventional Commits から自動的に生成
- `feat:` は「Features」セクションに表示
- `fix:` は「Bug Fixes」セクションに表示
- `BREAKING CHANGE:` は「Breaking Changes」セクションに表示
- CHANGELOG ファイルは管理しない（GitHub Releases 参照）

### ローカルでの確認

```bash
# 依存関係のインストール
pnpm install

# コミットメッセージの検証
pnpm commitlint --from HEAD~1 --to HEAD

# semantic-releaseのドライラン（実際には実行されない）
pnpm semantic-release --dry-run
```

### 必要なツール

- **mise**: Node.js と pnpm のバージョン管理（`.mise.toml`で定義）
- **pnpm**: パッケージマネージャー
- **semantic-release**: 自動リリースツール
- **commitlint**: コミットメッセージの検証

詳細は [`CONTRIBUTING.md`](CONTRIBUTING.md) を参照してください。

## 参考リンク

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Semantic Versioning 2.0.0（日本語）](https://semver.org/lang/ja/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Conventional Commits（日本語）](https://www.conventionalcommits.org/ja/)
- [semantic-release](https://github.com/semantic-release/semantic-release)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
