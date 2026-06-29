//! `clip` の CLI 全体の振る舞い（help / version / 不正引数 / 補完生成）。OS 非依存。

use predicates::prelude::*;
use rstest::rstest;

use crate::clip;

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    clip().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    clip()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("clip"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_subcommand_prints_help() {
    // command が Option になり、サブコマンド無しは help を表示して正常終了する。
    clip()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn unknown_subcommand_fails() {
    clip().arg("bogus").assert().failure().code(2);
}

#[rstest]
#[case("obj")]
#[case("text")]
#[case("path")]
fn subcommand_without_file_fails(#[case] sub: &str) {
    clip()
        .arg(sub)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required").or(predicate::str::contains("FILE")));
}

#[test]
fn completions_fish_prints_script() {
    clip()
        .args(["--completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("clip"));
}
