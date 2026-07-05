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
    /// Print a shell completion script to stdout and exit.
    #[arg(long, value_name = "SHELL")]
    completions: Option<Shell>,

    #[command(subcommand)]
    command: Option<Commands>,
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
}

/// CLI を解析し、サブコマンドへディスパッチする。
pub fn run() -> ExitCode {
    let cli = Cli::parse();

    // `--completions <shell>`: 補完スクリプトを stdout へ出して終了する。サブコマンドではなく
    // top-level option にすることで `clip <Tab>` の候補に出さない（fish は `-` 始まりのみ option を出す）。
    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "clip", &mut io::stdout());
        return ExitCode::SUCCESS;
    }

    match cli.command {
        Some(Commands::Obj { file }) => obj::run(&file),
        Some(Commands::Text { file }) => text::run(&file),
        Some(Commands::Path { file }) => path::run(&file),
        None => {
            let _ = Cli::command().print_help();
            ExitCode::SUCCESS
        }
    }
}
