//! `git-upstream` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! git 不在（PATH 差し替え）では「Git command not found → exit 8」の経路を
//! 決定的に検証する。CI / 開発機は非 root 前提（root の場合は先に exit 4）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

fn git_upstream() -> Command {
    Command::cargo_bin("git-upstream").unwrap()
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
