//! `clip text <file>`: ファイルの中身をクリップボードへコピーする（macOS 専用、
//! pbcopy 経由。旧 abbr `c-file`（`pbcopy < file`）相当）。

use std::process::ExitCode;

use crate::clipboard;

pub fn run(file: &str) -> ExitCode {
    if let Err(msg) = clipboard::ensure_macos() {
        eprintln!("clip text: {msg}");
        return ExitCode::FAILURE;
    }
    let data = match std::fs::read(file) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("clip text: not found: {file}");
            return ExitCode::FAILURE;
        }
    };
    match clipboard::copy_bytes(&data) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("clip text: {msg}");
            ExitCode::FAILURE
        }
    }
}
