//! git コマンド実行ラッパー。

use std::process::{Command, ExitCode};

/// `git diff --cached --name-only` のステージ済みファイル一覧。
pub(crate) fn staged_files() -> Vec<String> {
    git_capture(&["diff", "--cached", "--name-only"])
        .map(|out| {
            out.lines()
                .filter(|line| !line.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// `git <args>` の stdout を取得する。
pub(crate) fn git_capture(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `git <args>` を stdio 継承で実行し、終了コードを `ExitCode` で返す。
pub(crate) fn run_status(args: &[&str]) -> ExitCode {
    match Command::new("git").args(args).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::FAILURE,
    }
}

/// `git <args>` を stdio 継承で実行し、成功したかを返す。
pub(crate) fn git_status(args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
