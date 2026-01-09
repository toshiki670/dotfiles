# コントリビューションガイド

dotfiles プロジェクトへのコントリビューションをお考えいただき、ありがとうございます！

## 開発環境のセットアップ

### 必要なツール

このプロジェクトでは以下のツールを使用しています：

```bash
# Homebrew経由でインストール
brew install git gh zsh nvim mise sheldon
```

詳細は [`README.md`](README.md) を参照してください。

## コミット規約

このプロジェクトでは、わかりやすいコミットメッセージを推奨しています。
[Conventional Commits](https://www.conventionalcommits.org/ja/) 形式を使用することで、より明確な履歴を残すことができます（任意）。

### コミットメッセージの形式（推奨）

```
<type>(<scope>): <subject>

<body>

<footer>
```

### コミットタイプ（推奨）

- **feat**: 新機能
- **fix**: バグ修正
- **docs**: ドキュメントのみの変更
- **style**: コードの意味に影響しない変更（空白、フォーマットなど）
- **refactor**: バグ修正でも機能追加でもないコード変更
- **perf**: パフォーマンスを向上させるコード変更
- **test**: テストの追加・修正
- **chore**: ビルドプロセスやツールの変更
- **ci**: CI 設定ファイルの変更
- **build**: ビルドシステムや外部依存関係に影響する変更

**注意**: バージョン管理はリリース時に手動で選択するため、コミットメッセージの形式は強制されません。

### コミット例

#### 新機能の追加

```bash
git commit -m "feat: add new zsh completion for docker-compose"
```

#### バグ修正

```bash
git commit -m "fix: correct PATH order in zshrc"
```

#### 破壊的変更

```bash
git commit -m "feat!: migrate plugin manager from zinit to sheldon

BREAKING CHANGE: Users need to reinstall plugins after updating.
Run './install' script to apply changes."
```

#### ドキュメント更新

```bash
git commit -m "docs: update installation instructions in README"
```

### コミットメッセージのルール

1. **type**: 必須。上記のタイプから選択
2. **scope**: オプション。変更の範囲（zsh, vim, git など）
3. **subject**: 必須。変更の簡潔な説明
   - 小文字で始める
   - 末尾にピリオドを付けない
   - 命令形を使用（"add" not "added" or "adds"）
   - 100 文字以内
4. **body**: オプション。変更の詳細な説明
5. **footer**: オプション。破壊的変更や issue 参照

### コミットメッセージのヒント

わかりやすいコミットメッセージを心がけてください。Conventional Commits 形式は推奨されますが、強制ではありません。

## ブランチ戦略（GitHub Flow）

このプロジェクトは **GitHub Flow** を採用しています。

### 重要なルール

⚠️ **main ブランチへの直接コミット・プッシュは禁止**

すべての変更は Pull Request 経由で行います。

### ブランチ命名規則

- `feature/機能名` - 新機能の追加
- `fix/バグ名` - バグ修正
- `docs/ドキュメント名` - ドキュメント更新
- `refactor/対象` - リファクタリング
- `chore/タスク名` - ビルドタスクやツールの変更

### 開発フロー（GitHub Flow）

#### 1. main ブランチを最新に更新

```bash
git checkout main
git pull origin main
```

#### 2. 作業ブランチを作成

```bash
# 新機能の場合
git checkout -b feature/your-feature-name

# バグ修正の場合
git checkout -b fix/bug-description
```

#### 3. 変更を加える

```bash
# 変更を実施
vim zsh/.zshrc

# ステージング
git add zsh/.zshrc

# Conventional Commits形式でコミット
git commit -m "feat: add new alias for git operations"

# 複数のコミットを行ってもOK
git commit -m "test: add tests for new alias"
git commit -m "docs: update alias documentation"
```

#### 4. リモートにプッシュ

```bash
git push origin feature/your-feature-name
```

#### 5. Pull Request を作成

1. GitHub 上で Pull Request を作成
2. **Base branch**: `main`
3. PR のタイトルと説明を記入
4. レビューを依頼（該当する場合）

#### 6. レビューとテスト

- コードレビューを実施
- 必要に応じて修正を加える
- CI チェックが通ることを確認

#### 7. main ブランチにマージ

- Pull Request を承認
- **Squash and merge** を推奨（コミット履歴を整理）
- マージ後、必要に応じてリリースを実行（手動）

### マージ方法の推奨

**Squash and merge（推奨）:**

複数のコミットを 1 つにまとめてマージ。マージ時に Conventional Commits 形式のメッセージを記述。

```
feat: add new git aliases

- Add alias for git status
- Add alias for git log
- Update documentation
```

**通常の Merge:**

すべてのコミットが Conventional Commits 形式に従っている場合のみ使用。

## リリースプロセス

このプロジェクトは **手動リリース** を採用しています。

### リリースの実行方法

Pull Request を main ブランチにマージした後、エンジニアが手動でリリースを実行します。

**手順：**

1. GitHub リポジトリの「Actions」タブを開く
2. 「Release」ワークフローを選択
3. 「Run workflow」ボタンをクリック
4. **Release type** を選択：
   - **patch**: バグ修正（0.28.0 → 0.28.1）
   - **minor**: 新機能・破壊的変更（0.28.0 → 0.29.0）
5. 「Run workflow」を実行

**リリースアクションが実行すること：**

1. `bin/release`スクリプトが`VERSION`ファイルから現在のバージョンを読み取り
2. 選択されたリリースタイプに基づいてバージョンを計算
3. `VERSION`ファイルを更新してコミット
4. Git タグを作成（例: `v0.29.0`）
5. GitHub Release を作成（前回のタグからのコミットログを自動生成）

### リリースノートについて

- GitHub Releases の自動生成機能を使用
- CHANGELOG.md ファイルは管理しない
- リリースノートは GitHub Releases ページで確認

### バージョンアップのルール

手動で選択したリリースタイプに基づいてバージョンが決定されます：

- **patch**: バグ修正・小さな改善（0.28.0 → 0.28.1）
- **minor**: 新機能・破壊的変更（0.28.0 → 0.29.0）

### 複数の PR をまとめてリリース

複数の PR を main ブランチにマージしてから、任意のタイミングでリリースアクションを実行できます。

**例:**

```
PR#1: feat: add zsh completion (マージ)
PR#2: fix: PATH order bug (マージ)
PR#3: feat: add vim config (マージ)
→ エンジニアがリリースアクションを実行
→ v0.29.0 として一括リリース
```

**リリースタイミングの柔軟性：**

- 複数の PR をまとめて計画的にリリース
- 緊急のバグ修正は即座にリリース
- 機能が揃うまで待ってからリリース

詳細は [`VERSIONING.md`](VERSIONING.md) を参照してください。

### ローカルでのリリース確認

```bash
# 現在のバージョンを確認
cat VERSION

# リリーススクリプトのヘルプを表示
./bin/release -h

# ローカルでのリリース実行（対話モード）
./bin/release patch
./bin/release minor
```

## テスト

変更をコミットする前に、以下を確認してください：

1. **構文エラーがないこと**

```bash
# zshの構文チェック
zsh -n zsh/.zshrc

# vimの構文チェック（該当する場合）
vim -u NONE -c "source vim/config/setting.vim" -c "quit"
```

2. **実際の環境で動作すること**

```bash
# インストールスクリプトを実行
./install

# シェルを再起動
exec $SHELL -l
```

## コードスタイル

### Shell Script

- インデント: スペース 2 つ
- 引用符: 変数は原則としてダブルクォートで囲む
- シバン: `#!/bin/bash` または `#!/bin/zsh`

### Vim Script

- インデント: スペース 2 つ
- コメントは日本語 OK

### TOML

- インデント: スペース 2 つ
- セクションは用途ごとにグループ化

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
