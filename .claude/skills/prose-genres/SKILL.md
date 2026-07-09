---
name: prose-genres
description: "prose-tidy の観点3(経緯・実装事情を残すか削るか)を、このリポジトリの実際の文書ジャンルに当てはめて判定する。README.md / CONTRIBUTING.md / COLOR.md / commit message / PR・Issue本文・レビューコメント / fish・bash・TOML等のコード内平文コメントが対象。prose-tidy がこのリポジトリで発火したときに参照する。"
---

# prose-genres — このリポジトリの文書ジャンル別デフォルト

`prose-tidy` の観点3は「経緯を残すか削るかは文書の目的で決める」という手続きだけを定め、具体的な verdict はリポジトリごとの companion skill に委ねている。これはこのリポジトリ(dotfiles)での具体値。

## ジャンル別デフォルト

| ジャンル | 該当ファイル | 経緯・実装事情 |
| --- | --- | --- |
| 使い方ガイド/リファレンス | `README.md`、`COLOR.md` | 削る |
| ルール集 | `CONTRIBUTING.md` | 削る(ただし分類・判断基準として機能する理由は残す。例: 「`.claude/` 配下はホームに展開されないので `chore:`、展開されユーザー環境に影響するものは `feat:`」という基準そのもの) |
| 意思決定の記録 | commit message の body、PR/Issue 本文・レビューコメント | 残す |
| コード内平文コメント | `*.fish` / `*.sh` / `manifest.toml` 等のコメント | 削る(ただし Chesterton's Fence 予防は残す) |

Rust の `//!` / `///` および設計根拠を運ぶ隣接 `//` コメントは `rustdoc-tidy` の対象で、このジャンル表には含まれない。

## このリポジトリに無いジャンル

設計・内部構造の判断根拠は rustdoc に集約する方針(プロジェクト CLAUDE.md)のため、ADR のような独立した設計判断記録の文書ジャンルはこのリポジトリに存在しない。設計判断の経緯を書きたくなったら、まず「これは rustdoc の doc コメントに書くべきではないか」を疑う(該当すれば `rustdoc-tidy` の対象)。
