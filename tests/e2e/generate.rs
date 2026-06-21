//! `dotfiles apply` の generate 層（S2 / #456）の E2E。
//!
//! 実バイナリ（gh/bat 等）に依存せず、PATH 先頭に置いたスタブ（[`crate::write_stub`]）で
//! `cmd` 実行と deps gate を検証する。スタブは sh スクリプトなので unix 限定。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// generate 単位 `configs/foo/completion`（dst=ファイル / cmd=foo / deps=foo）を書き出す。
#[cfg(unix)]
fn write_generate_unit(work: &Path) -> std::path::PathBuf {
    let unit = work.join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\n\
         kind = \"generate\"\n\
         cmd = [\"foo\"]\n\
         deps = [\"foo\"]\n",
    )
    .unwrap();
    unit
}

/// kind=generate が `cmd` を実行し、その標準出力を dst のファイルへ書き出すことを検証する。
#[cfg(unix)]
#[test]
fn apply_generate_runs_cmd_and_writes_output() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'complete -c foo -f\\n'\n");
    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path()) // スタブのみ。foo は PATH 上で解決される。
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "complete -c foo -f\n",
        "cmd の stdout がそのまま dst に書かれていない",
    );
}

/// deps gate: 依存バイナリが PATH に無ければ生成をスキップし、ファイルを作らない（成功終了）。
#[cfg(unix)]
#[test]
fn apply_generate_gate_skips_when_dep_missing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // foo を置かない＝依存欠落。

    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("foo"));

    assert!(
        !home
            .path()
            .join(".config/fish/completions/foo.fish")
            .exists(),
        "gate が効かず依存欠落でも生成された",
    );
}

/// generate は単位内の `manifest.toml` 以外のファイル（gh の独自補完ブロック相当）を
/// 生成物の後ろへ連結する。
#[cfg(unix)]
#[test]
fn apply_generate_appends_sibling_files() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'GENERATED\\n'\n");
    let unit = write_generate_unit(work.path());
    fs::write(unit.join("custom.fish"), "# CUSTOM\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "GENERATED\n# CUSTOM\n",
        "生成物の後ろへ sibling が連結されていない",
    );
}

/// generate で `cmd` が無い manifest はエラー終了する。
#[test]
fn apply_generate_without_cmd_errors() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\nkind = \"generate\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cmd が必要"));
}

/// `dotfiles list` が generate 単位を generate ＋ deps 付きで表示することを検証する。
#[test]
fn list_shows_generate_kind_with_deps() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\n\
         kind = \"generate\"\n\
         cmd = [\"foo\"]\n\
         deps = [\"foo\"]\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("deps=foo"));
}
