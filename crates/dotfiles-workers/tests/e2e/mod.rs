//! `dotfiles-workers`（daily-check-worker / git-background-fetch-worker）の
//! E2E。worker は引数を取らず env 駆動のため、スタブや実 git repo を用いて
//! 副作用（結果ファイル / スタンプファイル作成）を検証する。

use assert_cmd::Command;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";
const DATE_STUB: &str = "#!/bin/sh\nprintf '2026-06-16\\n'\n";
const BREW_STUB: &str = "#!/bin/sh\nprintf 'pkg-a\\n'\n";
const MISE_STUB: &str = "#!/bin/sh\nprintf 'node@22\\n'\n";

fn write_exec(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn path_with(prefix: &Path) -> OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![prefix.to_path_buf()];
    paths.extend(std::env::split_paths(&existing));
    std::env::join_paths(paths).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn daily_check_worker_exits_success_without_tools() {
    Command::cargo_bin("daily-check-worker")
        .unwrap()
        .env("PATH", EMPTY_PATH)
        .assert()
        .success();
}

#[test]
fn daily_check_worker_writes_result_when_outdated_exists() {
    let bin = tempfile::tempdir().unwrap();
    let env = tempfile::tempdir().unwrap();
    let ts = env.path().join("daily.ts");
    let cache = env.path().join("cache");
    let result = env.path().join("daily.result");

    write_exec(bin.path(), "date", DATE_STUB);
    write_exec(bin.path(), "brew", BREW_STUB);
    write_exec(bin.path(), "mise", MISE_STUB);

    Command::cargo_bin("daily-check-worker")
        .unwrap()
        .env("PATH", path_with(bin.path()))
        .env("DAILY_CHECK_TS", &ts)
        .env("DAILY_CHECK_CACHE", &cache)
        .env("DAILY_CHECK_RESULT", &result)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&ts).unwrap(), "2026-06-16");
    let output = fs::read_to_string(&result).unwrap();
    assert!(output.contains("=== Homebrew Outdated Packages ==="));
    assert!(output.contains("pkg-a"));
    assert!(output.contains("=== Mise Outdated Tools ==="));
    assert!(output.contains("node@22"));
}

#[test]
fn git_background_fetch_worker_creates_throttle_stamp_in_repo() {
    let repo = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    git(repo.path(), &["init"]);

    Command::cargo_bin("git-background-fetch-worker")
        .unwrap()
        .current_dir(repo.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env("GIT_FETCH_THROTTLE_SEC", "9999")
        .assert()
        .success();

    let top = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo.path())
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .unwrap();
    assert!(top.status.success());
    let top = String::from_utf8_lossy(&top.stdout).trim().to_string();

    let hash = ProcessCommand::new("git")
        .arg("hash-object")
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(top.as_bytes())?;
            child.wait_with_output()
        })
        .unwrap();
    let id = String::from_utf8_lossy(&hash.stdout)
        .trim()
        .chars()
        .take(12)
        .collect::<String>();
    let stamp = cache.path().join("fish/git-fetch-last").join(id);
    assert!(stamp.is_file());
}
