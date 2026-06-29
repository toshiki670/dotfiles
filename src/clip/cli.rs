//! `clip` の CLI 定義（clap derive）とサブコマンドのディスパッチ。

use std::io;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use super::{obj, path, text};

/// ファイルをクリップボードへコピーする（macOS）。
#[derive(Parser)]
#[command(
    name = "clip",
    version,
    about = "Copy a file to the clipboard (obj / text / path; macOS)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Copy as a Finder-pasteable file object (macOS).
    Obj {
        /// コピー対象のファイル。
        file: String,
    },
    /// Copy the file's contents.
    Text {
        /// コピー対象のファイル。
        file: String,
    },
    /// Copy the absolute path (also printed to stdout).
    Path {
        /// コピー対象のファイル。
        file: String,
    },
    /// Print a shell completion script to stdout.
    Completions {
        /// 対象シェル（bash / fish / zsh / …）。
        shell: Shell,
    },
}

/// CLI を解析し、サブコマンドへディスパッチする。
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Obj { file } => obj::run(&file),
        Commands::Text { file } => text::run(&file),
        Commands::Path { file } => path::run(&file),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "clip", &mut io::stdout());
            ExitCode::SUCCESS
        }
    }
}
