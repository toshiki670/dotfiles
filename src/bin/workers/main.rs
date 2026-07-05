//! `workers`: fish の conf.d フックからバックグラウンドで起動される worker 群をまとめた bin。
//!
//! 各 worker（[`daily_check`] / [`git_background_fetch`]。詳細は各モジュールの doc を
//! 参照）は引数を受け取らず、環境変数で駆動する。`run()` は [`cli`] からのみ呼ばれる。
//! この `main()` が [`cli::run`] を呼ぶ入口。

use std::process::ExitCode;

mod cli;
mod daily_check;
mod git_background_fetch;

fn main() -> ExitCode {
    cli::run()
}
