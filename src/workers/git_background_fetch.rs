//! スロットル付き `git fetch` をバックグラウンドで実行する worker。
//! 旧 `_git_background_fetch_worker.fish` の移植。
//!
//! conf.d/70-git-background-fetch.fish の fish_postexec フックからバックグラウンドで起動される。
//! 任意で GIT_FETCH_THROTTLE_SEC を環境変数で受け取る（既定 20 秒）。

use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run() -> ExitCode {
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());

    // 現在のリポジトリのトップレベル。git 管理外なら何もしない。
    let top = match capture(
        "git",
        &["-C", &cwd.to_string_lossy(), "rev-parse", "--show-toplevel"],
    ) {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return ExitCode::SUCCESS,
    };

    let interval: u64 = std::env::var("GIT_FETCH_THROTTLE_SEC")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(20);

    let cache_base = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.cache")
    });
    let cache_root = format!("{cache_base}/fish/git-fetch-last");
    if std::fs::create_dir_all(&cache_root).is_err() {
        return ExitCode::SUCCESS;
    }

    // `echo -n $top | git hash-object --stdin` の先頭 12 文字。
    let id = match hash_object(&top) {
        Some(h) => h.chars().take(12).collect::<String>(),
        None => return ExitCode::SUCCESS,
    };
    let stamp_file = format!("{cache_root}/{id}");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // スロットル: 前回 fetch から interval 秒未満なら何もしない。
    if let Ok(last) = std::fs::read_to_string(&stamp_file)
        && let Ok(last) = last.trim().parse::<u64>()
        && now.saturating_sub(last) < interval
    {
        return ExitCode::SUCCESS;
    }

    let _ = std::fs::write(&stamp_file, now.to_string());

    let _ = Command::new("git")
        .args(["-C", &top, "fetch", "--quiet", "--no-write-fetch-head"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    ExitCode::SUCCESS
}

/// コマンドを実行して stdout を返す（失敗時 None）。
fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `git hash-object --stdin` に文字列を渡してハッシュを得る。
fn hash_object(content: &str) -> Option<String> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(content.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
