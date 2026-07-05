//! `fzf-picker` の CLI 定義とサブコマンドのディスパッチ。
//!
//! [`run`] が `main.rs` から呼ばれる入口。各サブコマンドの実体は対応モジュール
//! （[`super::cdabbr`] / [`super::fzf_gh`] / [`super::fzf_ghq_cd`] /
//! [`super::fzf_worktree_remove`]）にある。

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use super::{cdabbr, fzf_gh, fzf_ghq_cd, fzf_worktree_remove};

/// fzf 系ピッカーをまとめた入口。
#[derive(Parser)]
#[command(
    name = "fzf-picker",
    version,
    about = "fzf 系ピッカーをまとめたコマンド",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// prompt_pwd 風の省略パスを展開・選択し、選択先パスを stdout に出力する。
    Cdabbr {
        /// prompt_pwd 風の省略パス（`~` または `/` 始まり）。
        abbr_path: String,
    },
    /// Issue/PR を fzf で選び、`gh <group> <action> <number>` を stdout に出力する。
    FzfGh,
    /// ghq の repo/worktree を fzf で選び、選択先の絶対パスを stdout に出力する。
    FzfGhqCd,
    /// リンク worktree を fzf で選び、確認のうえ削除する。
    FzfWorktreeRemove,
}

/// `fzf-picker` の入口。サブコマンドへディスパッチする。
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Commands::Cdabbr { abbr_path } => cdabbr::run(abbr_path),
        Commands::FzfGh => fzf_gh::run(),
        Commands::FzfGhqCd => fzf_ghq_cd::run(),
        Commands::FzfWorktreeRemove => fzf_worktree_remove::run(),
    }
}
