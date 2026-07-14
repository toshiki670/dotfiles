//! 1 日 1 回 brew/mise の outdated を集計して結果ファイルに書き出すバックグラウンド worker。
//!
//! conf.d/60-daily-check.fish から、以下の環境変数を渡してバックグラウンドで起動される:
//!   DAILY_CHECK_TS / DAILY_CHECK_CACHE / DAILY_CHECK_RESULT

use std::process::{Command, ExitCode};

pub fn run() -> ExitCode {
    let ts = env_or_empty("DAILY_CHECK_TS");
    let cache = env_or_empty("DAILY_CHECK_CACHE");
    let result = env_or_empty("DAILY_CHECK_RESULT");

    let today = match capture("date", &["+%Y-%m-%d"]) {
        Some(s) => s.trim().to_string(),
        None => return ExitCode::SUCCESS,
    };

    // 当日実行済みなら何もしない。
    if let Ok(last) = std::fs::read_to_string(&ts)
        && last.trim_end() == today
    {
        return ExitCode::SUCCESS;
    }

    let _ = std::fs::create_dir_all(&cache);
    let _ = std::fs::write(&ts, &today);

    let brew_out = if command_exists("brew") {
        capture("brew", &["outdated"]).unwrap_or_default()
    } else {
        String::new()
    };
    let mise_out = if command_exists("mise") {
        capture("mise", &["outdated"]).unwrap_or_default()
    } else {
        String::new()
    };

    if brew_out.trim().is_empty() && mise_out.trim().is_empty() {
        return ExitCode::SUCCESS;
    }

    let mut lines: Vec<String> = vec!["=== Homebrew Outdated Packages ===".into(), String::new()];
    if !brew_out.trim().is_empty() {
        lines.extend(brew_out.lines().map(String::from));
        lines.push(String::new());
        lines.push(String::new());
    }
    if !mise_out.trim().is_empty() {
        lines.push("=== Mise Outdated Tools ===".into());
        lines.push(String::new());
        lines.extend(mise_out.lines().map(String::from));
        lines.push(String::new());
        lines.push(String::new());
    }

    let _ = std::fs::write(&result, lines.join("\n"));
    ExitCode::SUCCESS
}

fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

/// コマンドを実行して stdout を返す（失敗時 None）。
fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}
