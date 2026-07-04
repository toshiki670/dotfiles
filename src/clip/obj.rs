//! `clip obj <file>`: ファイルを Finder 貼り付け可能なファイルオブジェクトとして
//! クリップボードへコピーする（macOS 専用、osascript 経由）。

use std::process::ExitCode;

use super::clipboard;

pub fn run(file: &str) -> ExitCode {
    if let Err(msg) = clipboard::ensure_macos() {
        eprintln!("clip obj: {msg}");
        return ExitCode::FAILURE;
    }
    let path = match clipboard::resolve(file) {
        Ok(path) => path,
        Err(msg) => {
            eprintln!("clip obj: {msg}");
            return ExitCode::FAILURE;
        }
    };
    match clipboard::copy_file_object(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("clip obj: {msg}");
            ExitCode::FAILURE
        }
    }
}
