---
name: rustdoc-tidy
description: "Rust の doc コメント（//! と ///）を、rustdoc で実際に読まれる情報密度の観点でレビューし、無駄な記述を削る。設定/データファイル（manifest.toml 等）のコメントが rustdoc の焼き増しになっていないかも確認する。次のような依頼で使う: 「rustdoc を整理して」「doc コメントが冗長」「この括弧いる?」「crate の説明が事実の羅列」「モジュール一覧が重複してる」「旧実装への言及を消して」「doc コメントのジャーゴンを減らしたい」「manifest.toml のコメントが rustdoc と重複してる」。cargo doc / rustdoc / crate doc / module doc / 設定ファイルのコメントに触れる文脈でのみ発火する。"
---

# rustdoc-tidy — Rust doc コメントの無駄を削る

`//!`（crate/module doc）と `///`（item doc）を、**rustdoc で実際に公開される情報密度**の観点でレビューし、事実の羅列ではなく読者が必要とする最小限に絞る。コードの正しさは見ない（correctness は別の役目）。

進め方は `prose-tidy` skill に従い、重複・陳腐化・不要の判定も同じく `debt-audit` agent が行う。この skill が持つのは、**Rust では出所がどこにあるか**という知識だけ。

## 前提

- 対象は Rust の doc コメント（`//!` / `///`）。設計根拠を運ぶ場合に限り、隣接する平文 `//` コメントも対象にする。
- `--document-private-items` を使うプロジェクトでは private モジュールも公開ページに載るため、private/pub を問わず全 doc コメントがレビュー対象になりうる。
- **設定/データファイルのコメントも対象にする。** 構造（スキーマ）を Rust の型・struct の rustdoc で定義していることがあり（`serde::Deserialize` する struct の doc 等）、その設定ファイル自身にもコメントが付く（例: このプロジェクトの `configs/<tool>/manifest.toml`）。

## debt-audit へ渡す出所

`prose-tidy` の手順4で渡す材料に、次の2つを加える。

1. **rustdoc の自動生成物。** `cargo doc`（private を含めるプロジェクトなら `--document-private-items`）でビルドし、対象モジュールの `index.html` を実際に開く。rustdoc は `mod` 宣言から「Modules」表を各モジュール自身の doc 付きで自動生成するので、手書きのモジュール一覧はここが出所になる。孫モジュールの内訳のような、自動生成に出ないものはこの表に現れない。
2. **設定ファイルのスキーマを定義する側の rustdoc。** 設定ファイルのコメントに対して、そのスキーマを定義する struct の `///` と、その struct が属するモジュールの `//!` を開いて添える。属性の意味・合成のルール・条件分岐の仕組みといった一般仕様は rustdoc 側が出所になる。そのファイルだけの理由（なぜこの設定単位はこの値・この構成を選んだか、他の設定単位との依存関係）は rustdoc には書けないので、出所を持たない。

括弧書きには rustdoc 固有の例外がある。intra-doc リンクの括弧（`（[`crate::foo`]）`）とプラットフォーム制約の括弧（Unix のみ・非 Unix は no-op 等）は、消すと事実が失われることを渡すときに添える。

## 手順

1. スコープを決める（曖昧なら聞く）: 直前の diff だけか、crate/workspace 全体か。設定/データファイルまで含めるか。
2. `prose-tidy` の手順に沿って進め、手順4で渡す材料に上記の出所を加える。
3. 修正後: `cargo doc`（プロジェクトの利用フラグに合わせる）・`cargo test`・`cargo fmt --check`（プロジェクトの慣習に応じて）を通す。設定ファイルを直したときは、その設定を読むプログラムのテスト（あれば）も通す。

## 出力

`prose-tidy` の出力に、ビルド/テスト/fmt の結果を加える。
