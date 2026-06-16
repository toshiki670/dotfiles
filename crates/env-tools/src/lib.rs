//! 環境メンテナンス系コマンド（旧 fish/bash の glue）の共有ロジック。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ:
//!   - [`banner`] — セクション見出しの整形出力（純ロジック寄り）
//!   - [`command`] — 外部コマンドの存在確認とラベル付き実行（IO）
//!   - [`prompt`] — 対話 y/N 確認（純判定 `is_yes` ＋ stdin の IO ラッパ）
//! - **純ロジックと IO を分離**。判定（[`prompt::is_yes`]）はユニットテストを同居させ、
//!   外部コマンドを叩く IO 層（[`command`]）と対話 IO はバイナリの E2E（`tests/e2e/`）で
//!   検証する。
//! - **この lib はクレート内部限定**。未公開（crates.io 非掲載）なので他の配布クレート
//!   から path 依存してはいけない（release-plz `git_only` で `cargo package` が壊れる）。
//!   共有はクレート内の bin（`src/bin/*.rs`）からのみ行う。

pub mod banner;
pub mod command;
pub mod prompt;
