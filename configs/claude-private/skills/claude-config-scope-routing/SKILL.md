---
name: claude-config-scope-routing
description: "新しい Claude Code 設定（rule・skill・settings 等）を追加・変更するときに必ず使用する。`~/.claude/` 配下を直接変更する前に、読み込ませたい範囲と配布条件の2軸で追加先を判定する。"
---

# Claude Code 設定の追加先は2つの軸で決める

新しい Claude Code 設定（rule・skill・settings 等）は、**軸1: Claude Code 上で読み込ませたい範囲**と、**軸2: dotfiles がどの条件でそのマシンへ配布するか**を分けて決める。

## 手順

1. **軸1: 読み込ませたい範囲を決める。** 内容が、その場に居ることを前提とした参照（特定リポジトリ内のパスや他ルールのファイル名を、クローンの用意なしに辿らせる参照）を含むなら、そのリポジトリで作業するときだけ → そのリポジトリの `.claude/` 配下へ。含まないなら全リポジトリ → `configs/claude/` 配下へ。固有名詞への単なる言及は前提参照ではない。この判定は、追加しようとしている本文自身にも適用する。
2. **軸2: 配布条件を決める。** 無条件なら `configs/claude/` へそのまま追加する。条件付きなら `when`（`deps` / `os` / `profile`。詳細は `src/bin/dotfiles/manifest.rs` の doc comment）でゲートする。ユニット全体のゲートは独立ユニットで（例: この skill 自身を配る `configs/claude-private/`、`configs/yt/`）、`settings.json` パイプライン内の断片は step-level で（例: `configs/claude/settings/rtk.json`）。
3. **反映する。** クローンが無ければまず用意する（`gh repo clone toshiki670/dotfiles`）。対象ユニットへ追加し、`dotfiles apply` を実行する（home 全体への配置なので、ユーザーに確認してから）。コミットは、設定追加の依頼それ自体を許可とみなさず、通常のコミット規律に従う。
4. 迷うときは、自分で仮判定してからユーザーに確認する。

## 例外

一時的・実験的で、恒久化する価値がまだ無い設定は、dotfiles を経由せず `~/.claude/` へ直接書いてよい。
