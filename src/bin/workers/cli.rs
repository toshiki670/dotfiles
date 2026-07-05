//! `workers` の CLI 定義とサブコマンドのディスパッチ。
//!
//! [`run`] が `main.rs` から呼ばれる入口。各サブコマンドの実体は対応モジュール
//! （[`super::daily_check`] / [`super::git_background_fetch`]）にある。

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use super::{daily_check, git_background_fetch};

/// fish の conf.d フックから起動されるバックグラウンド worker をまとめた入口。
#[derive(Parser)]
#[command(
    name = "workers",
    version,
    about = "バックグラウンド worker をまとめたコマンド",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 1 日 1 回 brew/mise の outdated を集計して結果ファイルへ書く。
    DailyCheck,
    /// スロットル付き `git fetch` をバックグラウンドで実行する。
    GitBackgroundFetch,
}

/// `workers` の入口。サブコマンドへディスパッチする。
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Commands::DailyCheck => daily_check::run(),
        Commands::GitBackgroundFetch => git_background_fetch::run(),
    }
}
