//! `upkeep` の CLI 定義とサブコマンドのディスパッチ。
//!
//! [`run`] が `main.rs` から呼ばれる入口。各サブコマンドの実体は対応モジュール
//! （[`super::cleanup`] / [`super::doctor`] / [`super::upgrade`]）にある。

use std::io;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use super::{cleanup, doctor, outdated, upgrade};

#[derive(Parser)]
#[command(
    name = "upkeep",
    version,
    about = "環境メンテナンス系コマンドをまとめたコマンド",
    propagate_version = true
)]
struct Cli {
    /// Print a shell completion script to stdout and exit.
    #[arg(long, value_name = "SHELL")]
    completions: Option<Shell>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// パッケージマネージャのキャッシュ・不要バージョンを対話的に削除する。
    Cleanup {
        /// 実際には削除せず、削除対象だけを表示する。
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// brew / mise / cargo を一括更新する。
    Upgrade,
    /// brew / mise の健全性を診断する。
    Doctor,
    /// brew / mise / cargo でアップデート可能なパッケージを一覧表示する。
    Outdated {
        /// 取得できたリリースノートを claude -p で日本語要約する（機械的に解決できるのは基本的に cargo バイナリのみ）。
        #[arg(long)]
        explain: bool,
    },
}

/// CLI を解析し、サブコマンドへディスパッチする。
pub fn run() {
    let cli = Cli::parse();

    // `--completions <shell>`: 補完スクリプトを stdout へ出して終了する。サブコマンドではなく
    // top-level option にすることで `upkeep <Tab>` の候補に出さない（fish は `-` 始まりのみ option を出す）。
    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "upkeep", &mut io::stdout());
        return;
    }

    match cli.command {
        Some(Commands::Cleanup { dry_run }) => cleanup::run(dry_run),
        Some(Commands::Upgrade) => upgrade::run(),
        Some(Commands::Doctor) => doctor::run(),
        Some(Commands::Outdated { explain }) => outdated::run(explain),
        None => {
            let _ = Cli::command().print_help();
        }
    }
}
