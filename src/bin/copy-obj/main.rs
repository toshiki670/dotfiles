//! ファイルをファイルオブジェクトとしてクリップボードへコピーする（Finder で
//! 貼り付け可能、macOS 専用）。旧 `copy-obj.fish` の移植。

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    if std::env::consts::OS != "macos" {
        eprintln!("copy-obj: macOS only");
        return ExitCode::FAILURE;
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: copy-obj <file>");
        return ExitCode::FAILURE;
    }

    // path resolve + test -e 相当（実在しなければ canonicalize が失敗する）。
    let abspath = match std::fs::canonicalize(&args[0]) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("copy-obj: not found: {}", args[0]);
            return ExitCode::FAILURE;
        }
    };

    let script = format!("set the clipboard to POSIX file \"{}\"", abspath.display());
    match Command::new("osascript").arg("-e").arg(script).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::FAILURE,
    }
}
