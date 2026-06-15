//! `gh-clone` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! gh / ghq 不在は PATH を実在しないディレクトリに差し替えて再現する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

fn gh_clone() -> Command {
    Command::cargo_bin("gh-clone").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    gh_clone().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    gh_clone()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("gh-clone"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_required_repo_arg_fails_with_usage() {
    gh_clone()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required").or(predicate::str::contains("REPO")));
}

#[test]
fn missing_gh_exits_127() {
    gh_clone()
        .arg("owner/repo")
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .code(127)
        .stderr(predicate::str::contains("gh command not found"));
}
