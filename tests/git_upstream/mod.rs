//! `git-upstream` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 実 git リポジトリを使って initialize / merge の挙動を検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

fn git_upstream() -> Command {
    Command::cargo_bin("git-upstream").unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

fn git_capture(dir: &Path, args: &[&str]) -> String {
    let out = ProcessCommand::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed in {}",
        dir.display()
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

struct Fixture {
    _tmp: TempDir,
    seed: std::path::PathBuf,
    upstream_bare: std::path::PathBuf,
    local: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let seed = root.join("seed");
        let upstream_bare = root.join("upstream.git");
        let local = root.join("local");

        let _ = ProcessCommand::new("git")
            .args(["init", "-b", "master"])
            .arg(&seed)
            .status();
        git(&seed, &["config", "user.name", "E2E"]);
        git(&seed, &["config", "user.email", "e2e@example.com"]);
        fs::write(seed.join("README.md"), "seed\n").unwrap();
        git(&seed, &["add", "README.md"]);
        git(&seed, &["commit", "-m", "chore: seed"]);

        let status = ProcessCommand::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(&seed)
            .arg(&upstream_bare)
            .status()
            .unwrap();
        assert!(status.success());

        let status = ProcessCommand::new("git")
            .arg("clone")
            .arg(&seed)
            .arg(&local)
            .status()
            .unwrap();
        assert!(status.success());
        git(&local, &["config", "user.name", "E2E"]);
        git(&local, &["config", "user.email", "e2e@example.com"]);

        Self {
            _tmp: tmp,
            seed,
            upstream_bare,
            local,
        }
    }

    fn push_new_upstream_commit(&self) {
        let work = self.seed.parent().unwrap().join("upstream-work");
        let status = ProcessCommand::new("git")
            .arg("clone")
            .arg(&self.upstream_bare)
            .arg(&work)
            .status()
            .unwrap();
        assert!(status.success());
        git(&work, &["config", "user.name", "E2E"]);
        git(&work, &["config", "user.email", "e2e@example.com"]);
        fs::write(work.join("upstream.txt"), "upstream change\n").unwrap();
        git(&work, &["add", "upstream.txt"]);
        git(&work, &["commit", "-m", "feat: upstream"]);
        git(&work, &["push", "origin", "master"]);
    }
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    git_upstream().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    git_upstream()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("git-upstream"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_git_exits_8() {
    git_upstream()
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .code(8)
        .stderr(predicate::str::contains("Git command not found"));
}

#[test]
fn initialize_adds_upstream_remote_and_fetches() {
    let fx = Fixture::new();
    git_upstream()
        .current_dir(&fx.local)
        .arg("-i")
        .arg(&fx.upstream_bare)
        .assert()
        .success();

    let remotes = git_capture(&fx.local, &["remote"]);
    assert!(remotes.lines().any(|line| line.trim() == "upstream"));
}

#[test]
fn merge_fetches_upstream_master_into_local() {
    let fx = Fixture::new();
    git(
        &fx.local,
        &[
            "remote",
            "add",
            "upstream",
            &fx.upstream_bare.to_string_lossy(),
        ],
    );
    fx.push_new_upstream_commit();
    let before = git_capture(&fx.local, &["rev-parse", "HEAD"]);

    git_upstream().current_dir(&fx.local).assert().success();

    let after = git_capture(&fx.local, &["rev-parse", "HEAD"]);
    assert_ne!(before, after);
    assert!(fx.local.join("upstream.txt").is_file());
}
