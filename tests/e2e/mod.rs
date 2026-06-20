//! root `dotfiles`（core）bin の E2E テスト（assert_cmd + predicates + rstest）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::fs;
use std::path::Path;

fn dotfiles() -> Command {
    Command::cargo_bin("dotfiles").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    dotfiles().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    dotfiles()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("dotfiles"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_args_prints_version() {
    dotfiles()
        .assert()
        .success()
        .stdout(predicate::str::contains("dotfiles"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// `dotfiles apply` が固定ソース `configs/` を読み、`manifest.toml`（dst / kind=copy 既定）
/// に従って一時 HOME へ実体を配置することを検証する（S0 / #454 の受け入れ条件）。
/// 実ソースである repo の `configs/zellij` をそのまま使い、configs 化が機能することを確かめる。
#[test]
fn apply_places_real_zellij_config_into_home() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let src = Path::new(repo_root).join("configs/zellij/config.kdl");
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(repo_root)
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/zellij/config.kdl");
    assert!(
        placed.is_file(),
        "zellij config が配置されていない: {placed:?}"
    );
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        fs::read_to_string(&src).unwrap(),
        "配置された内容がソースと一致しない",
    );
}

/// kind 省略時に copy として扱われ、`~` が HOME に展開されることを、
/// 一時ソース fixture で検証する（hermetic）。
#[test]
fn apply_defaults_to_copy_and_expands_tilde() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    // 一時ソース configs/demo/{manifest.toml, hello.conf} を用意（kind は省略）。
    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(unit.join("manifest.toml"), "dst = \"~/.config/demo\"\n").unwrap();
    fs::write(unit.join("hello.conf"), "hello = 1\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/hello.conf");
    assert!(placed.is_file(), "fixture が配置されていない: {placed:?}");
    assert_eq!(fs::read_to_string(&placed).unwrap(), "hello = 1\n");
    // manifest.toml 自体は配置対象外。
    assert!(
        !home.path().join(".config/demo/manifest.toml").exists(),
        "manifest.toml が誤って配置された",
    );
}

/// `configs/` が無い場所で apply するとエラー終了することを検証する。
#[test]
fn apply_errors_when_source_missing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ソースが見つからない"));
}
