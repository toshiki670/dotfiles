//! `v-sync` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! nvim / chezmoi 不在を再現するため、PATH を実在しないディレクトリに差し替えて
//! 「コマンド未検出 → exit 127」の経路を決定的に検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;

/// PATH 上に何も存在しない状態を作るための実在しないディレクトリ。
const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

fn v_sync() -> Command {
    Command::cargo_bin("v-sync").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    v_sync().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    v_sync()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("v-sync"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_nvim_exits_127() {
    v_sync()
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .code(127)
        .stderr(predicate::str::contains("nvim command not found"));
}
