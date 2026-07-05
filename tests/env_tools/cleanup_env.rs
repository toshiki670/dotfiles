//! `cleanup-env` の E2E（実バイナリ + スタブ PM で検証）。
//!
//! 検証: `--help`/`--version`、確認 `y` で実削除コマンドを呼ぶ、`--dry-run` で各コマンドに
//! dry-run フラグが付く、確認 `n`／EOF（非対話）で何も実行しない、未知オプションで失敗。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, write_stubs};

fn cleanup_env() -> Command {
    let mut cmd = Command::cargo_bin("env-tools").unwrap();
    cmd.arg("cleanup-env");
    cmd
}

/// brew / mise / cargo / cargo-cache のスタブと呼び出しログを用意する。
struct Fixture {
    _root: TempDir,
    bin: PathBuf,
    log: PathBuf,
}

fn fixture() -> Fixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    write_stubs(&bin, &["brew", "mise", "cargo", "cargo-cache"]);
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
    cleanup_env().arg(flag).assert().success();
}

#[test]
fn confirm_yes_runs_real_cleanups() {
    let fx = fixture();

    cleanup_env()
        .env("PATH", &fx.bin)
        .env("ENV_TOOLS_LOG", &fx.log)
        .write_stdin("y\ny\ny\ny\n") // brew cleanup / brew autoremove / mise prune / cargo cache
        .assert()
        .success();

    let log = log_of(&fx);
    assert!(log.contains("brew cleanup"), "missing brew cleanup:\n{log}");
    assert!(
        log.contains("brew autoremove"),
        "missing brew autoremove:\n{log}"
    );
    assert!(log.contains("mise prune"), "missing mise prune:\n{log}");
    assert!(
        log.contains("cargo cache --autoclean"),
        "missing cargo cache:\n{log}"
    );
    assert!(
        !log.contains("--dry-run"),
        "no dry-run flag expected:\n{log}"
    );
}

#[test]
fn dry_run_passes_dry_flags() {
    let fx = fixture();

    cleanup_env()
        .arg("--dry-run")
        .env("PATH", &fx.bin)
        .env("ENV_TOOLS_LOG", &fx.log)
        .write_stdin("y\ny\ny\ny\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(dry-run: nothing will actually be removed)",
        ));

    let log = log_of(&fx);
    assert!(
        log.contains("brew cleanup --dry-run"),
        "missing brew cleanup --dry-run:\n{log}"
    );
    assert!(
        log.contains("brew autoremove --dry-run"),
        "missing brew autoremove --dry-run:\n{log}"
    );
    assert!(
        log.contains("mise prune --dry-run"),
        "missing mise prune --dry-run:\n{log}"
    );
    assert!(
        log.contains("cargo cache --dry-run --autoclean"),
        "missing cargo cache --dry-run --autoclean:\n{log}"
    );
}

#[test]
fn confirm_no_runs_nothing() {
    let fx = fixture();

    cleanup_env()
        .env("PATH", &fx.bin)
        .env("ENV_TOOLS_LOG", &fx.log)
        .write_stdin("n\nn\nn\nn\n")
        .assert()
        .success();

    assert!(!fx.log.exists(), "nothing should run when answering no");
}

#[test]
fn eof_stdin_runs_nothing() {
    // 非対話（stdin が即 EOF）では各確認が No 扱いになり何も実行しない。
    let fx = fixture();

    cleanup_env()
        .env("PATH", &fx.bin)
        .env("ENV_TOOLS_LOG", &fx.log)
        .write_stdin("")
        .assert()
        .success();

    assert!(!fx.log.exists(), "nothing should run on EOF stdin");
}

#[test]
fn unknown_option_fails() {
    cleanup_env()
        .arg("--bogus")
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure();
}
