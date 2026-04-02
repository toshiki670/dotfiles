---
description: Never commit on main; use a feature branch first (enforced for all git work)
---

# Git — do not commit on `main`

`main`（およびリポジトリの default ブランチ）に**直接コミットしない**。ローカルだけでも禁止。

## 必須フロー

1. 変更の前またはコミットの直前に `git branch --show-current` で現在ブランチを確認する。
2. いま `main`（または `master`、リモートの default と同一名）なら、**先に**作業ブランチを切る: `git checkout -b <type>/[<id>-]<short-description>`。
   - `<id>` は Issue・チケット番号など（例: `123`）。存在する場合のみ付与し、`-` で description と繋ぐ。
   - 例: `feat/42-add-login`、`fix/PROJECT-8-null-check`、`chore/update-deps`
3. そのブランチ上でだけ `git add` / `git commit` / `git push` を行う。

## 例外

ユーザーが**明示的に**「このコミットは main に直接」などと指示した場合のみ許可する。
