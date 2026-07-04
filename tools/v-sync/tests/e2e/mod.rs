//! `v-sync` の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 外部コマンド `nvim` は PATH 先頭に置くスタブで差し替え、同期手順
//! （nvim headless 実行 -> configs/nvim への書き戻し）を実バイナリで検証する。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// PATH 上に何も存在しない状態を作るための実在しないディレクトリ。
const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";
const NVIM_STUB: &str =
    "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"$NVIM_ARGS_FILE\"\nexit \"${NVIM_EXIT:-0}\"\n";

fn v_sync() -> Command {
    Command::cargo_bin("v-sync").unwrap()
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
    home: TempDir,
    bin: TempDir,
    repo: TempDir,
    out: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        write_exec(bin.path(), "nvim", NVIM_STUB);
        fs::write(repo.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::create_dir_all(repo.path().join("configs/nvim")).unwrap();
        Self {
            home,
            bin,
            repo,
            out,
        }
    }

    fn path(&self) -> OsString {
        path_with(self.bin.path())
    }

    fn nvim_args_file(&self) -> PathBuf {
        self.out.path().join("nvim.args")
    }

    fn repo_lock(&self) -> PathBuf {
        self.repo.path().join("configs/nvim/lazy-lock.json")
    }

    fn write_live_lock(&self, content: &str) {
        let home_lock = self.home.path().join(".config/nvim/lazy-lock.json");
        fs::create_dir_all(home_lock.parent().unwrap()).unwrap();
        fs::write(&home_lock, content).unwrap();
    }
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

#[test]
fn runs_nvim_sync_then_writes_lock_into_configs() {
    let fx = Fixture::new();
    fx.write_live_lock("{\"synced\":true}\n");

    v_sync()
        .current_dir(fx.repo.path())
        .env("PATH", fx.path())
        .env("HOME", fx.home.path())
        .env("NVIM_ARGS_FILE", fx.nvim_args_file())
        .assert()
        .success()
        .stdout(predicate::str::contains("v-sync: syncing nvim plugins"))
        .stdout(predicate::str::contains(
            "v-sync: writing lazy-lock.json back into configs/nvim",
        ));

    assert_eq!(
        fs::read_to_string(fx.nvim_args_file()).unwrap(),
        "--headless\n+Lazy! sync\n+qa\n"
    );
    assert_eq!(
        fs::read_to_string(fx.repo_lock()).unwrap(),
        "{\"synced\":true}\n"
    );
}

#[test]
fn missing_live_lock_fails() {
    let fx = Fixture::new();

    v_sync()
        .current_dir(fx.repo.path())
        .env("PATH", fx.path())
        .env("HOME", fx.home.path())
        .env("NVIM_ARGS_FILE", fx.nvim_args_file())
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to write"));
}
