# バージョニングルール

このドキュメントでは、dotfiles プロジェクトのバージョニングルールを定義します。

## セマンティックバージョニング 0.x.x

このプロジェクトは[セマンティックバージョニング 2.0.0](https://semver.org/)に従い、**0.x.x 形式**でバージョン管理を行います。

### バージョン形式

```text
v0.MINOR.PATCH
```

- **MAJOR**: 常に 0（固定）
- **MINOR**: 破壊的変更を含む機能追加・変更
- **PATCH**: バグ修正や小さな改善

### MAJOR を 0 に固定する理由

dotfiles は開発スタイルや時代の変化に応じて前後の依存関係を無視した変更を継続的に加えていく性質のため、MAJOR を 1 以上に上げる予定はない。破壊的変更は常に MINOR バージョンで表現する。

## リリースの実行

リリースフローと実行手順は [`CONTRIBUTING.md`](CONTRIBUTING.md#リリースプロセス) を参照してください。

リリースアクションが実行すること：

1. 最新の GitHub リリースから現在のバージョンを取得
2. `bin/bump_version` でバージョンを計算
3. Git タグを作成（`v` プレフィックス付き）
4. GitHub Release を作成（リリースノート自動生成）

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

## 参考リンク

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Semantic Versioning 2.0.0（日本語）](https://semver.org/lang/ja/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Conventional Commits（日本語）](https://www.conventionalcommits.org/ja/)
- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
