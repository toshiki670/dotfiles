//! `copy-obj` の E2E テスト（assert_cmd + predicates + rstest）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

fn copy_obj() -> Command {
    Command::cargo_bin("copy-obj").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    copy_obj().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    copy_obj()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("copy-obj"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_required_file_arg_fails_with_usage() {
    copy_obj()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required").or(predicate::str::contains("FILE")));
}
