//! クリップボードへ書き込む低レベル手段（pbcopy / osascript）と、各サブコマンドが
//! 共有する macOS ガード・パス解決。すべて macOS 専用。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// macOS 以外なら `Err`（メッセージは呼び出し側で `clip <sub>: …` の形で表示する）。
pub fn ensure_macos() -> Result<(), &'static str> {
    if std::env::consts::OS == "macos" {
        Ok(())
    } else {
        Err("macOS only")
    }
}

/// 実在するファイルの絶対パスを返す（`path resolve` + 存在チェック相当）。
pub fn resolve(file: &str) -> Result<PathBuf, String> {
    std::fs::canonicalize(file).map_err(|_| format!("not found: {file}"))
}

/// バイト列を pbcopy でクリップボードへ書き込む。
pub fn copy_bytes(data: &[u8]) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run pbcopy: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "failed to open pbcopy stdin".to_string())?
        .write_all(data)
        .map_err(|e| format!("failed to write to pbcopy: {e}"))?;
    let status = child.wait().map_err(|e| format!("pbcopy failed: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("pbcopy exited with error".to_string())
    }
}

/// 文字列を pbcopy でコピーする。
pub fn copy_text(text: &str) -> Result<(), String> {
    copy_bytes(text.as_bytes())
}

/// ファイルを Finder 貼り付け可能なファイルオブジェクトとしてコピーする（osascript）。
pub fn copy_file_object(path: &Path) -> Result<(), String> {
    let script = format!("set the clipboard to POSIX file \"{}\"", path.display());
    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|e| format!("failed to run osascript: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("osascript exited with error".to_string())
    }
}
