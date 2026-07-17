# Claude Code 設定の追加先は2つの軸で決める

新しい Claude Code 設定（rule・skill・settings 等）は、**軸1: Claude Code 上で読み込ませたい範囲**と、**軸2: dotfiles がどの条件でそのマシンへ配布するか**を分けて決める。**`~/.claude/` 配下を直接変更する前に、必ずこの判定を行う。**新規ファイルの作成に限らず、`~/.claude/settings.json` のような既存ファイルへの追記も対象になる。

固有名詞（個人ツール名等）への言及は軸1を決めない。`configs/claude/settings/rtk.json` は `when.deps = ["rtk"]` を使い、rtk 未検出の環境ではその断片だけを無視しつつ全リポジトリへ配布されている——「特定ツールへの言及があるから project-scoped」という判定は誤り。ツール依存性は軸2（配置条件）の問題であり、軸1（Claude Code 上の適用範囲）とは別軸。

## 必須フロー

0. **まず現物の前例を見る。** このリポジトリには既に `configs/claude/skills/`・`.claude/skills/`（このリポジトリ限定）・`configs/claude-private/`（profile 限定、このファイル自身が例）が並存している。`ls` 等で直接確認し、追加しようとしている物がどれに近いか照合する。前例を見ずに「似たジャンルだから」で既存の配置先へ機械的に追随しない。
1. **軸1: Claude Code 上の適用範囲を判定する。**
   - **全リポジトリ**（内容そのものが、特定リポジトリ名・特定の他ルールのファイル名を参照しない、汎用的な原則やワークフローであること） → `configs/claude/` 配下（具体的な配置先は軸2で決める）。
   - **`toshiki670/dotfiles` リポジトリで作業するときだけ**（内容が dotfiles リポジトリ自身の構造・コマンド・別ルールを参照する等、そのリポジトリ固有の知識を前提にする） → そのリポジトリ自身の配下に、種類に応じて置く。
     - rule: `.claude/rules/*.md`
     - skill: `.claude/skills/*/SKILL.md`
     - settings（チームと共有してよい内容）: `.claude/settings.json`
     - settings（自分だけ・このリポジトリだけに留めたい内容）: `.claude/settings.local.json`（`.gitignore` 登録済みだが、まだ実ファイルは存在しない）
2. **軸2: 「全リポジトリ」を選んだ場合、どの条件でマシンへ配布するかを決める。**
   - **全 dotfiles 環境に無条件** → 既存の `configs/claude/rules/` や `configs/claude/skills/` にそのまま追加する。
   - **特定の profile を選んだマシンだけ** → `configs/yt/manifest.toml`・`configs/claude-private/manifest.toml`（このファイル自身）と同じパターンで、`configs/claude/` とは別の独立ユニットを作り、unit-level `when = { profile = "..." }` を付け、同じ `output = "~/.claude"` へ向ける（既存 `configs/claude/` ユニットに `when` を足すと、そのユニットの全ファイルが道連れでゲートされる。`output = "~/.config/fish/conf.d"` を複数ユニットが分担している前例と同じ要領）。
   - **特定 OS だけ** → 同様に、独立ユニットで unit-level `when.os` を使う。
   - **特定ツールが PATH にあるときだけ** → `rtk.json` と同じパターン。既存の settings.json パイプラインに載せるなら step-level `when.deps`、rule/skill 単体なら独立ユニットで unit-level `when.deps` を検討する。
   - **dotfiles を経由しないローカル設定**（一時的・実験的で、恒久化する価値がまだ無い） → `~/.claude/` へ直接書く。
   - `when` 語彙自体の詳細（`deps` / `os` / `profile` の意味・書く位置）は `src/bin/dotfiles/manifest.rs` の doc comment が出所。ここでは繰り返さない。
3. **配置を決める前に、書いた内容自体にこの判定を適用する（自己適用チェック）。** 本文を読み返し、軸1の失格基準（特定のリポジトリ名〔例: `toshiki670/dotfiles`〕・そのリポジトリ内にしか存在しないパス・ファイル名〔他ルールのファイル名等〕）を、**その場に居ることを前提として**参照していないか確認する。`~/.claude/...` のような一般的なパスの参照、特定ツール名への言及（軸2で扱う）、そして読者がどこで読んでもクローンの用意から案内できる外部への誘導（例:「詳しい手順は自分の dotfiles クローンの `.claude/rules/<name>.md` に従う」のように、クローンが無ければ用意する前提で書かれた誘導）はこれに該当しない。1つでも前提参照に該当すれば「全リポジトリ」は失格で、「`toshiki670/dotfiles` リポジトリで作業するときだけ」へ回す。
4. 「全リポジトリ」を選んだ場合、対象ユニットへの追加だけでは `~/.claude/` へ反映されない。
   - クローンが無ければ、まず用意する（`gh repo clone toshiki670/dotfiles` 等）。
   - ブランチは現在の Git 状態とユーザーの依頼に従って決める。既存の作業ブランチに含めるのが適切ならそれに従い、無関係な既存作業を汚すなら `main` から独立した worktree を切る。
   - 対象ユニット（`configs/claude/` または軸2で決めた独立ユニット）配下に追加する。
   - `dotfiles apply` を実行して `~/.claude/` へ反映する（home 全体への配置行為なのでユーザーに確認してから実行する）。
   - コミットするかどうかは、設定追加の依頼それ自体を許可とみなさず、通常のコミット規律（ユーザーの明示的な依頼、`git-no-commit-on-main`）に従う。
5. 軸1・軸2のどれに当たるか迷うときは、自分で仮判定してからユーザーに確認する。

## 例外

なし。
