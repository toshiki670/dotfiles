---
name: release-prepare
description: "Release Prepare workflow（release-plz の release-pr）を起動し、完了を待って作成・更新された Release PR と次の version を報告する。`mise run release-prepare` の skill 版。このリポジトリ(dotfiles)限定。外部に影響する操作なので `/release-prepare` で明示的に呼ばれたときだけ使う。"
disable-model-invocation: true
---

# release-prepare — Release PR を作成・更新する

`.github/workflows/release-prepare.yml`（`workflow_dispatch` のみ）を起動して、release-plz に Release PR（`release-*` ブランチ）を作らせる。リリースフロー全体は [CONTRIBUTING.md](../../../CONTRIBUTING.md#リリースフロー) を参照。

`mise run release-prepare` は `read` で y/N の入力を待つので、Claude からは呼ばない。`/release-prepare` で呼ばれたこと自体を実行の承認とみなす。

## 手順

`gh` が使える環境ならそれを使う。使えない環境（cloud セッション等）では GitHub MCP の `actions_run_trigger` / `actions_list` / `actions_get` / `list_pull_requests` で同じことをする。

1. workflow を起動する。

   ```bash
   gh workflow run release-prepare.yml --repo toshiki670/dotfiles --ref main
   ```

2. 起動した run を特定して完了まで待つ。起動直後は一覧に出ないことがあるので、数秒置いてから取得する。

   ```bash
   gh run list --repo toshiki670/dotfiles --workflow release-prepare.yml --event workflow_dispatch --limit 1 \
     --json databaseId,status,url,createdAt
   gh run watch <databaseId> --repo toshiki670/dotfiles --exit-status
   ```

   失敗したら `gh run view <databaseId> --repo toshiki670/dotfiles --log-failed` で失敗したステップのログを確認し、原因を添えて報告して終わる（再実行はしない）。

3. Release PR を探す。

   ```bash
   gh pr list --repo toshiki670/dotfiles --state open --json number,title,url,headRefName \
     --jq '.[] | select(.headRefName | startswith("release-"))'
   ```

4. 結果を報告する。
   - PR がある: PR の URL と次の version（PR タイトル、または PR の `Cargo.toml` の差分から読む）、新規作成か既存 PR の更新かを伝える。
   - PR が無い: 前回のタグ以降に `release-plz.toml` の `release_commits`（`feat` / `fix` / `perf` / `refactor` / `revert`）に当たるコミットが無く、bump 対象が無かったと考えられる。`git log $(git describe --tags --abbrev=0)..origin/main --oneline` で確認して、その旨を伝える。

Release PR のマージ（＝リリース実行）はこの skill の範囲外。ユーザーが自分で行う。
