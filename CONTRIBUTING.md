# コントリビューションガイド

dotfilesプロジェクトへのコントリビューションをお考えいただき、ありがとうございます！

## 開発環境のセットアップ

### 必要なツール

このプロジェクトでは以下のツールを使用しています：

```bash
# Homebrew経由でインストール
brew install mise

# miseでNode.jsとpnpmをインストール（プロジェクトルートで実行）
cd ~/dotfiles
mise install
```

### 依存関係のインストール

```bash
# プロジェクトの依存関係をインストール
pnpm install
```

## コミット規約

このプロジェクトは [Conventional Commits](https://www.conventionalcommits.org/ja/) に従います。

### コミットメッセージの形式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### コミットタイプ

- **feat**: 新機能（MINOR バージョンアップをトリガー）
- **fix**: バグ修正（PATCH バージョンアップをトリガー）
- **docs**: ドキュメントのみの変更
- **style**: コードの意味に影響しない変更（空白、フォーマットなど）
- **refactor**: バグ修正でも機能追加でもないコード変更
- **perf**: パフォーマンスを向上させるコード変更
- **test**: テストの追加・修正
- **chore**: ビルドプロセスやツールの変更
- **ci**: CI設定ファイルの変更
- **build**: ビルドシステムや外部依存関係に影響する変更

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
   - 100文字以内
4. **body**: オプション。変更の詳細な説明
5. **footer**: オプション。破壊的変更やissue参照

### コミットメッセージの検証

コミット前に自動的に検証されます：

```bash
# 手動で検証する場合
pnpm commitlint --from HEAD~1 --to HEAD
```

## ブランチ戦略

### ブランチ命名規則

- `feature/機能名` - 新機能の追加
- `fix/バグ名` - バグ修正
- `docs/ドキュメント名` - ドキュメント更新
- `refactor/対象` - リファクタリング

### 開発フロー

1. **ブランチを作成**

```bash
git checkout -b feature/your-feature-name
```

2. **変更を加える**

```bash
# 変更を実施
vim zsh/.zshrc

# ステージング
git add zsh/.zshrc

# Conventional Commits形式でコミット
git commit -m "feat: add new alias for git operations"
```

3. **プッシュ**

```bash
git push origin feature/your-feature-name
```

4. **Pull Requestを作成**

GitHub上でPull Requestを作成してください。

## リリースプロセス

このプロジェクトは **完全自動リリース** を採用しています。

### 自動リリースの仕組み

1. mainブランチにマージ
2. GitHub Actionsが自動起動
3. semantic-releaseがコミットメッセージを解析
4. バージョン番号を自動決定
5. CHANGELOGを自動生成・更新
6. Gitタグを作成（例: `v0.29.0`）
7. GitHub Releaseを作成
8. package.jsonを更新

### バージョンアップのルール

- `feat:` コミット → **MINOR** アップ（0.28.0 → 0.29.0）
- `fix:` コミット → **PATCH** アップ（0.28.0 → 0.28.1）
- `BREAKING CHANGE` → **MINOR** アップ（0.x.x系では）
- その他 → バージョン変更なし

詳細は [`VERSIONING.md`](VERSIONING.md) を参照してください。

### ローカルでのリリース確認

```bash
# ドライラン（実際には実行されない）
pnpm semantic-release --dry-run
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

- インデント: スペース2つ
- 引用符: 変数は原則としてダブルクォートで囲む
- シバン: `#!/bin/bash` または `#!/bin/zsh`

### Vim Script

- インデント: スペース2つ
- コメントは日本語OK

### TOML

- インデント: スペース2つ
- セクションは用途ごとにグループ化

## ヘルプ・質問

質問や提案がある場合は、GitHubのIssueを作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。

