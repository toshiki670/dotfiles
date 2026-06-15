//! ファイルをファイルオブジェクトとしてクリップボードへコピーする（Finder で
//! 貼り付け可能、macOS 専用）。旧 `copy-obj.fish` の移植。

use std::process::{Command, ExitCode};

use clap::Parser;

/// ファイルを Finder で貼り付け可能なファイルオブジェクトとしてコピーする（macOS）。
#[derive(Parser)]
#[command(
    name = "copy-obj",
    version,
    about = "Copy a file as a Finder-pasteable file object (macOS)"
)]
struct Cli {
    /// コピー対象のファイル。
    file: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if std::env::consts::OS != "macos" {
        eprintln!("copy-obj: macOS only");
        return ExitCode::FAILURE;
    }

    // path resolve + test -e 相当（実在しなければ canonicalize が失敗する）。
    let abspath = match std::fs::canonicalize(&cli.file) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("copy-obj: not found: {}", cli.file);
            return ExitCode::FAILURE;
        }
    };

    let script = format!("set the clipboard to POSIX file \"{}\"", abspath.display());
    match Command::new("osascript").arg("-e").arg(script).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::FAILURE,
    }
}
