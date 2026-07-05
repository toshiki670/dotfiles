//! `env-tools` の CLI 定義とサブコマンドのディスパッチ。
//!
//! [`run`] が `main.rs` から呼ばれる入口。各サブコマンドの実体は対応モジュール
//! （[`super::cleanup_env`] / [`super::upgrade_env`]）にある。

use clap::{Parser, Subcommand};

use super::{cleanup_env, upgrade_env};

/// 環境メンテナンス系コマンドをまとめた入口。
#[derive(Parser)]
#[command(
    name = "env-tools",
    version,
    about = "環境メンテナンス系コマンドをまとめたコマンド",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// パッケージマネージャのキャッシュ・不要バージョンを対話的に削除する。
    CleanupEnv {
        /// 実際には削除せず、削除対象だけを表示する。
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// brew / mise / cargo を一括更新する。
    UpgradeEnv,
}

/// `env-tools` の入口。サブコマンドへディスパッチする。
pub fn run() {
    match Cli::parse().command {
        Commands::CleanupEnv { dry_run } => cleanup_env::run(dry_run),
        Commands::UpgradeEnv => upgrade_env::run(),
    }
}
