//! `color` の E2E テスト（assert_cmd + predicates + rstest）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

fn color() -> Command {
    Command::cargo_bin("color").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    color().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    color()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("color"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn prints_16_and_256_color_tables() {
    color()
        .assert()
        .success()
        .stdout(predicate::str::contains("16 Colors"))
        .stdout(predicate::str::contains("256 Colors"));
}
