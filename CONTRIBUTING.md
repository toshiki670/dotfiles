# コントリビューションガイド

## 開発環境のセットアップ

### 必要なツール

```bash
# Homebrew経由でインストール
brew install git gh mise fish
```

lint/format ツール群（shfmt / shellcheck / taplo / stylua / rumdl / ruff）と
Rust・Node.js ツールチェーンは、リポジトリの `mise.toml` から `mise install` で供給されます
（`fish` のみ brew）。textlint 等の node devDependencies は `npm install` で供給されます。

dotfiles 以外のプロジェクトでも textlint を使いたい場合は、グローバルインストールが別途必要です:

```bash
npm install -g textlint textlint-rule-preset-ja-technical-writing textlint-rule-preset-ai-writing
```

`dotfiles apply` は `~/.textlintrc.json`(一般的な日本語/Markdown 向け preset)を配置しますが、
textlint 本体・preset パッケージの導入は上記の手動コマンドが担います。

dotfiles の利用に必要なツールは [`README.md`](README.md) を参照してください。

## コミット規約

[Conventional Commits](https://www.conventionalcommits.org/ja/) 形式を必須とする。**PR title も Conventional Commits 形式にすること**。

### このリポジトリ固有の判断基準

- `.claude/` 配下のスキルや `tools/` 配下の開発・保守ツール等、dotfiles 内部でのみ使われるツール・スクリプトの追加・変更は `chore:` を使う（`feat:` ではない）。
  - `feat:` はホームディレクトリに展開され、ユーザー環境に影響する設定・機能の追加に使う。

## ブランチ戦略

**GitHub Flow** を採用。ブランチ命名: `<type>/[<id>-]<short-description>`

- `<id>` は Issue・チケット番号（例: `42`、`PROJECT-8`）。存在する場合のみ付与し、`-` で繋ぐ。
- 例: `feat/42-add-login`、`fix/PROJECT-8-null-check`、`chore/update-deps`

## リリースプロセス

このリポジトリは [release-plz](https://release-plz.dev/) を使用して、Conventional Commits に基づくバージョン管理を自動化しています。version の source of truth は root `Cargo.toml`、CHANGELOG は git-cliff（`cliff.toml`）で生成します。crates.io へは publish せず、git タグ + GitHub Release のみを作成します。

**単一版**: 配布物は root `dotfiles` パッケージ 1 つに統合済みで、版もこの 1 つだけを振ります。タグは `v{version}`（従来どおり）、CHANGELOG は root `CHANGELOG.md` に生成されます。`tools/` 配下の開発・保守ツール（`dotfiles-lint` / `v-sync`）は `release = false` で release-plz の対象外（版を振りません）。下表のバンプ規則は、その版に触れたコミットの type で判定されます。

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
3. Release PR を `main` にマージすると Release Publish workflow が走り、タグ `v{version}` と GitHub Release が作成される。

`main` への直接 push は不要・不可。Release PR のマージがリリースのトリガー（ブランチ保護と両立）。

### 現在のバージョン確認

```bash
# 配布物 dotfiles（単一版）
git describe --tags --abbrev=0 --match 'v[0-9]*'
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

### テスト方針（エンジンはツールのライフサイクルから独立）

`dotfiles` バイナリは汎用エンジンで、`configs/` の個々のツール（claude / bat / ghostty …）はいつか消えるデータ。テストは「configs から特定ツールを削除・改名しても壊れない」ことを原則とする（壊れるなら defect として直す）。2 層で書く:

| 層 | 目的 | 入力 |
| --- | --- | --- |
| 契約テスト | エンジンの挙動（生成方式 × 合成 × `when` gate …）を固定する | hermetic な架空 fixture（`faketool` 等）。実 configs を名指ししない |
| 実 configs の妥当性確認 | 実際の `configs/` が manifest スキーマ・不変条件を満たすか | `configs/` の全ユニットを data-driven に走査する（ツール名をハードコードしない） |

原則はここに一度だけ明文化する。テストコード側は本節を参照し、再記述しない。

## コードスタイル

`mise run lint` で適用される linter / formatter に従う。

## ヘルプ・質問

質問や提案がある場合は、GitHub の Issue を作成してください。

## ライセンス

このプロジェクトへのコントリビューションは、[MIT License](LICENSE)の下でライセンスされます。
