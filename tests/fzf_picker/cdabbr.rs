//! `cdabbr` の E2E（実バイナリを起動して検証）。
//!
//! 検証: `--help`/`--version`、引数なし(clap で 2)、相対パス拒否、該当なしで失敗、
//! fzf 不在で単一候補はそのまま出力・再帰展開・複数候補は一覧して失敗、fzf ありで
//! 選択行を出力。`HOME` を一時ディレクトリに差し替えて展開対象を制御する。

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, FZF_STUB, path_with, write_exec};

fn cdabbr() -> Command {
    let mut cmd = Command::cargo_bin("fzf-picker").unwrap();
    cmd.arg("cdabbr");
    cmd
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn meta_flags_succeed(#[case] flag: &str) {
    cdabbr().arg(flag).assert().success();
}

#[test]
fn missing_arg_is_clap_error() {
    cdabbr().assert().failure().code(2);
}

#[test]
fn relative_path_rejected() {
    cdabbr()
        .arg("foo/bar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must start with ~ or /"));
}

#[test]
fn no_matching_path_fails() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join("alpha")).unwrap();
    cdabbr()
        .arg("~/z")
        .env("HOME", home.path())
        .env("PATH", EMPTY_PATH) // fzf 不在
        .assert()
        .failure()
        .stderr(predicate::str::contains("no matching path"));
}

#[test]
fn single_match_without_fzf_prints_path() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join("documents")).unwrap();
    let expected = home.path().join("documents");
    cdabbr()
        .arg("~/d")
        .env("HOME", home.path())
        .env("PATH", EMPTY_PATH)
        .assert()
        .success()
        .stdout(format!("{}\n", expected.display()));
}

#[test]
fn recursive_single_match_without_fzf() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join("aaa/bbb")).unwrap();
    let expected = home.path().join("aaa/bbb");
    cdabbr()
        .arg("~/a/b")
        .env("HOME", home.path())
        .env("PATH", EMPTY_PATH)
        .assert()
        .success()
        .stdout(format!("{}\n", expected.display()));
}

#[test]
fn multiple_matches_without_fzf_lists_and_fails() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join("dev")).unwrap();
    fs::create_dir_all(home.path().join("docs")).unwrap();
    cdabbr()
        .arg("~/d")
        .env("HOME", home.path())
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .stderr(predicate::str::contains("multiple matches"));
}

#[test]
fn fzf_selection_prints_choice() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join("dev")).unwrap();
    fs::create_dir_all(home.path().join("docs")).unwrap();
    let bin = TempDir::new().unwrap();
    let dump = bin.path().join("fzf-input.txt");
    write_exec(bin.path(), "fzf", FZF_STUB);
    let chosen = home.path().join("dev");

    cdabbr()
        .arg("~/d")
        .env("HOME", home.path())
        .env("PATH", path_with(bin.path()))
        .env("FZF_DUMP", &dump)
        .env("FZF_PICK", chosen.display().to_string())
        .assert()
        .success()
        .stdout(format!("{}\n", chosen.display()));

    // dev / docs の 2 候補が fzf へ渡る。
    let candidates = fs::read_to_string(&dump).unwrap_or_default();
    assert_eq!(
        candidates.lines().count(),
        2,
        "two candidates expected:\n{candidates}"
    );
}
