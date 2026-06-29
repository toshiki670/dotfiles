//! `clip path <file>`: ファイルの絶対パスをクリップボードへコピーし、確認のため
//! stdout にも出力する（macOS 専用、pbcopy 経由。旧 abbr `c-path`
//! （`path resolve | pbcopy; echo (pbpaste)`）相当）。

use std::process::ExitCode;

use super::clipboard;

pub fn run(file: &str) -> ExitCode {
    if let Err(msg) = clipboard::ensure_macos() {
        eprintln!("clip path: {msg}");
        return ExitCode::FAILURE;
    }
    let path = match clipboard::resolve(file) {
        Ok(path) => path,
        Err(msg) => {
            eprintln!("clip path: {msg}");
            return ExitCode::FAILURE;
        }
    };
    let text = path.display().to_string();
    match clipboard::copy_text(&text) {
        Ok(()) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("clip path: {msg}");
            ExitCode::FAILURE
        }
    }
}
