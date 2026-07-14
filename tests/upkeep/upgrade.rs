//! `upgrade` の E2E（実バイナリ + スタブ PM で検証）。
//!
//! 検証: `--help`/`--version`、全 PM 存在で brew/mise/cargo を順に更新呼び出し、PM 不在
//! （空 PATH）で何も呼ばず見出しだけ、`cargo` はあるが `cargo-install-update` 不在で Cargo
//! ステップをスキップ。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, write_stubs};

fn upgrade() -> Command {
    let mut cmd = Command::cargo_bin("upkeep").unwrap();
    cmd.arg("upgrade");
    cmd
}

/// スタブ PM 群と呼び出しログを用意する。
struct Fixture {
    _root: TempDir,
    bin: PathBuf,
    log: PathBuf,
}

fn fixture(stubs: &[&str]) -> Fixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    write_stubs(&bin, stubs);
    let log = root.path().join("calls.log");
    Fixture {
        _root: root,
        bin,
        log,
    }
}

fn log_of(fx: &Fixture) -> String {
    fs::read_to_string(&fx.log).unwrap_or_default()
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn meta_flags_succeed(#[case] flag: &str) {
    upgrade().arg(flag).assert().success();
}

#[test]
fn upgrades_all_present_managers_in_order() {
    let fx = fixture(&["brew", "mise", "cargo", "cargo-install-update"]);

    upgrade()
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success()
        .stdout(predicate::str::contains("Upgrading Environment"))
        .stdout(predicate::str::contains("Done"));

    let log = log_of(&fx);
    assert!(log.contains("brew upgrade"), "missing brew upgrade:\n{log}");
    assert!(log.contains("mise upgrade"), "missing mise upgrade:\n{log}");
    assert!(log.contains("mise reshim"), "missing mise reshim:\n{log}");
    assert!(
        log.contains("cargo install-update --locked --all"),
        "missing cargo install-update:\n{log}"
    );
}

#[test]
fn skips_absent_managers() {
    let fx = fixture(&[]); // スタブなし
    upgrade()
        .env("PATH", EMPTY_PATH) // どの PM も見えない
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success()
        .stdout(predicate::str::contains("Upgrading Environment"))
        .stdout(predicate::str::contains("=== ").not()); // 実行された PM セクションなし

    assert!(!fx.log.exists(), "no command should have run");
}

#[test]
fn skips_cargo_when_install_update_missing() {
    // cargo はあるが cargo-install-update がない → Cargo ステップはスキップ。
    let fx = fixture(&["cargo"]);

    upgrade()
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success();

    let log = log_of(&fx);
    assert!(
        !log.contains("install-update"),
        "Cargo step should be skipped:\n{log}"
    );
}
