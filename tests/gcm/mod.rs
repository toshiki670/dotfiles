//! `gcm` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 外部コマンド `claude` は PATH 先頭に置くスタブで差し替え、`git` は実物を使う
//! （PATH に実 PATH を残す）。一時リポジトリを `git init` してステージ済み差分から
//! 実際に commit されることを検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";
const CLAUDE_STUB: &str = "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' \"$CLAUDE_JSON\"\n";

fn gcm() -> Command {
    Command::cargo_bin("gcm").unwrap()
}

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

fn git_capture(dir: &Path, args: &[&str]) -> String {
    let out = ProcessCommand::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

struct Fixture {
    repo: TempDir,
    bin: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let repo = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        git(repo.path(), &["init"]);
        git(repo.path(), &["config", "user.name", "E2E"]);
        git(repo.path(), &["config", "user.email", "e2e@example.com"]);
        git(
            repo.path(),
            &["commit", "--allow-empty", "-m", "chore: init"],
        );
        write_exec(bin.path(), "claude", CLAUDE_STUB);

        Self { repo, bin }
    }

    fn repo_path(&self) -> &Path {
        self.repo.path()
    }

    fn stub_path(&self) -> PathBuf {
        self.bin.path().to_path_buf()
    }
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    gcm().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    gcm()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("gcm"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_staged_changes_fails() {
    gcm()
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ステージされた変更がありません"));
}

#[test]
fn commits_single_proposal_with_real_git_repo() {
    let fx = Fixture::new();
    let file = fx.repo_path().join("a.txt");
    fs::write(&file, "hello\n").unwrap();
    git(fx.repo_path(), &["add", "a.txt"]);

    gcm()
        .current_dir(fx.repo_path())
        .env("PATH", path_with(&fx.stub_path()))
        .env(
            "CLAUDE_JSON",
            "[{\"message\":\"feat: add alpha\",\"files\":[\"a.txt\"]}]",
        )
        .write_stdin("\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("feat: add alpha"));

    assert_eq!(
        git_capture(fx.repo_path(), &["log", "-1", "--pretty=%s"]),
        "feat: add alpha"
    );
    assert!(
        git_capture(
            fx.repo_path(),
            &["show", "--name-only", "--pretty=format:", "HEAD"]
        )
        .lines()
        .any(|line| line == "a.txt")
    );
}

#[test]
fn commits_multiple_proposals_as_split_commits() {
    let fx = Fixture::new();
    fs::write(fx.repo_path().join("a.txt"), "alpha\n").unwrap();
    fs::write(fx.repo_path().join("b.txt"), "beta\n").unwrap();
    git(fx.repo_path(), &["add", "a.txt", "b.txt"]);

    gcm()
        .current_dir(fx.repo_path())
        .env("PATH", path_with(&fx.stub_path()))
        .env(
            "CLAUDE_JSON",
            r#"[{"message":"feat: add alpha","files":["a.txt"]},{"message":"fix: add beta","files":["b.txt"]}]"#,
        )
        .write_stdin("\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("提案されたコミット (2 件)"));

    assert_eq!(
        git_capture(fx.repo_path(), &["log", "-1", "--pretty=%s"]),
        "fix: add beta"
    );
    assert_eq!(
        git_capture(fx.repo_path(), &["log", "-2", "--pretty=%s"]),
        "fix: add beta\nfeat: add alpha"
    );
    assert!(
        git_capture(
            fx.repo_path(),
            &["show", "--name-only", "--pretty=format:", "HEAD"]
        )
        .lines()
        .any(|line| line == "b.txt")
    );
    assert!(
        git_capture(
            fx.repo_path(),
            &["show", "--name-only", "--pretty=format:", "HEAD~1"]
        )
        .lines()
        .any(|line| line == "a.txt")
    );
}
