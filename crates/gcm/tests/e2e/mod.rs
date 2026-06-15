//! `gcm` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! git 不在（PATH 差し替え）ではステージ済みファイルが空とみなされ、
//! 「ステージされた変更がありません」で失敗する経路を決定的に検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

fn gcm() -> Command {
    Command::cargo_bin("gcm").unwrap()
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
