//! 環境メンテナンス系コマンド。共有ロジックと各 bin の `run()`。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ:
//!   - [`banner`] — セクション見出しの整形出力（純ロジック寄り）
//!   - [`command`] — 外部コマンドの存在確認とラベル付き実行（IO）
//!   - [`prompt`] — 対話 y/N 確認（純判定 `is_yes` ＋ stdin の IO ラッパ）
//! - **純ロジックと IO を分離**。判定（[`prompt::is_yes`]）はユニットテストを同居させ、
//!   外部コマンドを叩く IO 層（[`command`]）と対話 IO はバイナリの E2E（`tests/env_tools/`）で
//!   検証する。
//! - 各 bin（[`cleanup_env`] / [`upgrade_env`]）は `run()` を公開し、
//!   `src/bin/<name>.rs` の数行シムから呼ばれる。

pub mod banner;
pub mod command;
pub mod prompt;

pub mod cleanup_env;
pub mod upgrade_env;
