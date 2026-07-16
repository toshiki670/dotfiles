//! `doctor` の E2E（実バイナリ + スタブ PM で検証）。
//!
//! 検証: `--help`/`--version`、全 PM 存在で順に診断呼び出し、PM 不在（空 PATH）でスキップ、
//! 診断が問題を検出（非ゼロ exit）しても `upkeep doctor` 自体は成功しその出力を表示する。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, write_exec};

fn doctor() -> Command {
    let mut cmd = Command::cargo_bin("upkeep").unwrap();
    cmd.arg("doctor");
    cmd
}

/// スタブ PM 群と呼び出しログを用意する。
struct Fixture {
    _root: TempDir,
    bin: PathBuf,
    log: PathBuf,
}

fn fixture() -> Fixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let log = root.path().join("calls.log");
    Fixture {
        _root: root,
        bin,
        log,
    }
}

fn healthy_stub(name: &str) -> String {
    format!("#!/bin/sh\nprintf '{name} %s\\n' \"$*\" >> \"$UPKEEP_LOG\"\n")
}

fn unhealthy_stub(name: &str) -> String {
    format!(
        "#!/bin/sh\nprintf '{name} %s\\n' \"$*\" >> \"$UPKEEP_LOG\"\necho '{name} has issues' >&2\nexit 1\n"
    )
}

fn log_of(fx: &Fixture) -> String {
    fs::read_to_string(&fx.log).unwrap_or_default()
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn meta_flags_succeed(#[case] flag: &str) {
    doctor().arg(flag).assert().success();
}

#[test]
fn diagnoses_all_present_managers_in_order() {
    let fx = fixture();
    write_exec(&fx.bin, "brew", &healthy_stub("brew"));
    write_exec(&fx.bin, "mise", &healthy_stub("mise"));

    doctor()
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success()
        .stdout(predicate::str::contains("Diagnosing Environment"))
        .stdout(predicate::str::contains("Done"));

    let log = log_of(&fx);
    assert!(log.contains("brew doctor"), "missing brew doctor:\n{log}");
    assert!(log.contains("mise doctor"), "missing mise doctor:\n{log}");
}

#[test]
fn skips_absent_managers() {
    let fx = fixture();

    doctor()
        .env("PATH", EMPTY_PATH)
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success()
        .stdout(predicate::str::contains("Diagnosing Environment"))
        .stdout(predicate::str::contains("=== ").not());

    assert!(!fx.log.exists(), "no command should have run");
}

#[test]
fn unhealthy_manager_reports_but_does_not_block() {
    let fx = fixture();
    write_exec(&fx.bin, "brew", &unhealthy_stub("brew"));

    doctor()
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .assert()
        .success() // 問題があっても upkeep doctor 自体の exit code は 0
        .stdout(predicate::str::contains(
            "Homebrew doctor failed, continuing",
        ))
        .stderr(predicate::str::contains("brew has issues")); // 子プロセス自身の診断出力
}
