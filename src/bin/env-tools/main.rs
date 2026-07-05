//! `env-tools`: 環境メンテナンス系コマンドをまとめた bin。共有ロジックと各サブコマンドの
//! `run()`。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ（[`banner`] /
//!   [`command`] / [`prompt`]。詳細は各モジュールの doc を参照）。
//! - **純ロジックと IO を分離**。判定（[`prompt::is_yes`]）はユニットテストを同居させ、
//!   外部コマンドを叩く IO 層（[`command`]）と対話 IO はバイナリの E2E（`tests/env_tools/`）で
//!   検証する。
//! - 各サブコマンド（`cleanup-env` / `upgrade-env`）の `run()` は [`cli`] からのみ
//!   呼ばれる。この `main()` が [`cli::run`] を呼ぶ入口。

mod banner;
mod command;
mod prompt;

mod cleanup_env;
mod cli;
mod upgrade_env;

fn main() {
    cli::run();
}
