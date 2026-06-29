//! `gh-clone` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 外部コマンド `gh` / `ghq` は PATH 先頭に置くスタブで差し替え、clone -> migrate ->
//! 移設先パス出力（stdout）を実バイナリで検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";
const GH_STUB: &str = "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"$GH_ARGS_FILE\"\nrepo=\"${3##*/}\"\nmkdir -p \"$repo\"\nprintf 'cloned %s\\n' \"$3\"\n";
const GHQ_STUB: &str = "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"$GHQ_ARGS_FILE\"\nrepo=\"$3\"\ndst=\"$GHQ_MIGRATED_ROOT/$repo\"\nmkdir -p \"$dst\"\nprintf '%s\\n' \"$dst\"\n";

fn gh_clone() -> Command {
    Command::cargo_bin("gh-clone").unwrap()
}

fn write_exec(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn path_with(prefix: &Path) -> OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![prefix.to_path_buf()];
    paths.extend(std::env::split_paths(&existing));
    std::env::join_paths(paths).unwrap()
}

struct Fixture {
    work: TempDir,
    bin: TempDir,
    migrated_root: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let work = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let migrated_root = tempfile::tempdir().unwrap();
        write_exec(bin.path(), "gh", GH_STUB);
        write_exec(bin.path(), "ghq", GHQ_STUB);
        Self {
            work,
            bin,
            migrated_root,
        }
    }

    fn path(&self) -> OsString {
        path_with(self.bin.path())
    }

    fn gh_args_file(&self) -> PathBuf {
        self.work.path().join("gh.args")
    }

    fn ghq_args_file(&self) -> PathBuf {
        self.work.path().join("ghq.args")
    }
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    gh_clone().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    gh_clone()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("gh-clone"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_required_repo_arg_fails_with_usage() {
    gh_clone()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required").or(predicate::str::contains("REPO")));
}

#[test]
fn missing_gh_exits_127() {
    gh_clone()
        .arg("owner/repo")
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .code(127)
        .stderr(predicate::str::contains("gh command not found"));
}

#[test]
fn clones_then_migrates_and_prints_migrated_path() {
    let fx = Fixture::new();
    let expected = fx.migrated_root.path().join("repo");

    gh_clone()
        .current_dir(fx.work.path())
        .arg("owner/repo")
        .env("PATH", fx.path())
        .env("GH_ARGS_FILE", fx.gh_args_file())
        .env("GHQ_ARGS_FILE", fx.ghq_args_file())
        .env("GHQ_MIGRATED_ROOT", fx.migrated_root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected.display().to_string()))
        .stderr(predicate::str::contains("cloned owner/repo"));

    assert_eq!(
        fs::read_to_string(fx.gh_args_file()).unwrap(),
        "repo\nclone\nowner/repo\n"
    );
    assert_eq!(
        fs::read_to_string(fx.ghq_args_file()).unwrap(),
        "migrate\n-y\nrepo\n"
    );
    assert!(expected.is_dir());
}
