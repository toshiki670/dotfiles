//! `outdated` の E2E（実バイナリ + スタブ PM/外部コマンドで検証）。
//!
//! 検証: `--help`/`--version`、PM 個別/複数該当時の一覧表示、該当なしメッセージ、
//! `cargo` はあるが `cargo-install-update` 不在で cargo ステップをスキップ、
//! `--explain`（claude 不在で警告フォールバック、cargo 対象の要約成功、brew/mise 対象は
//! 「変更内容不明」、`gh release view` 失敗時の「変更内容不明」、claude 生成失敗時の
//! 「要約失敗」）。
//!
//! 外部コマンド（brew/mise/cargo/curl/gh/claude）は環境変数の中身をそのまま stdout に
//! 返すスタブ（[`stdout_stub_body`]）で差し替える。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, stdout_stub_body, stub_body, write_exec};

const BREW_JSON: &str = r#"{"formulae":[{"name":"bat","installed_versions":["0.24.0"],"current_version":"0.25.0","pinned":false,"pinned_version":null}],"casks":[]}"#;
const MISE_JSON: &str = r#"{"jq":{"name":"jq","requested":"1.6","current":"1.6","bump":"1.8","latest":"1.8.2","source":{"type":"mise.toml","path":"/tmp/mise.toml"}}}"#;
const CARGO_TABLE: &str =
    "Package      Installed  Latest   Needs update\ncargo-audit  v0.17.0    v0.18.0  Yes";
const CRATES_IO_JSON: &str = r#"{"crate":{"repository":"https://github.com/rustsec/rustsec"}}"#;
const GH_RELEASE_JSON: &str = r#"{"body":"What's Changed\n\n* Fix bug X","url":"https://github.com/rustsec/rustsec/releases/tag/v1.0.0"}"#;
const CLAUDE_SUMMARY_JSON: &str =
    r#"{"type":"result","is_error":false,"structured_output":{"summary":"新機能Xを追加"}}"#;
const CLAUDE_ERROR_JSON: &str = r#"{"is_error":true,"errors":["boom"]}"#;
const FAILING_STUB: &str = "#!/bin/sh\nexit 1\n";

fn outdated() -> Command {
    let mut cmd = Command::cargo_bin("upkeep").unwrap();
    cmd.arg("outdated");
    cmd
}

struct Fixture {
    _root: TempDir,
    bin: PathBuf,
}

fn fixture() -> Fixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    Fixture { _root: root, bin }
}

impl Fixture {
    /// `name` を、環境変数 `env_var` の中身をそのまま返すスタブとして置く。
    fn stub_stdout(&self, name: &str, env_var: &str) -> &Self {
        write_exec(&self.bin, name, &stdout_stub_body(env_var));
        self
    }

    /// `name` を任意のスタブ本体で置く（[`FAILING_STUB`] 等）。
    fn stub(&self, name: &str, body: &str) -> &Self {
        write_exec(&self.bin, name, body);
        self
    }

    /// `cargo` に加えて、存在確認だけされる `cargo-install-update` を置く。
    fn cargo_stub(&self) -> &Self {
        self.stub_stdout("cargo", "CARGO_TABLE")
            .stub("cargo-install-update", &stub_body("cargo-install-update"))
    }
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn meta_flags_succeed(#[case] flag: &str) {
    outdated().arg(flag).assert().success();
}

#[test]
fn no_updates_available() {
    outdated()
        .env("PATH", EMPTY_PATH)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "アップデート可能なものはありません",
        ));
}

#[test]
fn lists_brew_only() {
    let fx = fixture();
    fx.stub_stdout("brew", "BREW_JSON");

    outdated()
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("[brew] bat: 0.24.0 -> 0.25.0"));
}

#[test]
fn lists_mise_only() {
    let fx = fixture();
    fx.stub_stdout("mise", "MISE_JSON");

    outdated()
        .env("PATH", &fx.bin)
        .env("MISE_JSON", MISE_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("[mise] jq: 1.6 -> 1.8.2"));
}

#[test]
fn lists_cargo_only() {
    let fx = fixture();
    fx.cargo_stub();

    outdated()
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[cargo] cargo-audit: v0.17.0 -> v0.18.0",
        ));
}

#[test]
fn skips_cargo_when_install_update_missing() {
    let fx = fixture();
    fx.stub_stdout("cargo", "CARGO_TABLE");
    // cargo-install-update を置かない。

    outdated()
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "アップデート可能なものはありません",
        ));
}

#[test]
fn lists_all_three_sources() {
    let fx = fixture();
    fx.stub_stdout("brew", "BREW_JSON")
        .stub_stdout("mise", "MISE_JSON")
        .cargo_stub();

    outdated()
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("MISE_JSON", MISE_JSON)
        .env("CARGO_TABLE", CARGO_TABLE)
        .assert()
        .success()
        .stdout(predicate::str::contains("[brew] bat"))
        .stdout(predicate::str::contains("[mise] jq"))
        .stdout(predicate::str::contains("[cargo] cargo-audit"));
}

#[test]
fn explain_without_claude_warns_and_falls_back() {
    let fx = fixture();
    fx.stub_stdout("brew", "BREW_JSON");
    // claude を置かない。

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("[brew] bat: 0.24.0 -> 0.25.0"))
        .stdout(predicate::str::contains("要約").not())
        .stderr(predicate::str::contains("claude コマンドが見つかりません"));
}

#[test]
fn explain_summarizes_cargo_package() {
    let fx = fixture();
    fx.cargo_stub();
    fx.stub_stdout("curl", "CRATES_IO_JSON")
        .stub_stdout("gh", "GH_RELEASE_JSON")
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("GH_RELEASE_JSON", GH_RELEASE_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("要約: 新機能Xを追加"))
        .stdout(predicate::str::contains(
            "出典: https://github.com/rustsec/rustsec/releases/tag/v1.0.0",
        ));
}

#[test]
fn explain_shows_unavailable_for_non_cargo_source() {
    let fx = fixture();
    fx.stub_stdout("brew", "BREW_JSON")
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("変更内容不明"));
}

#[test]
fn explain_shows_unavailable_when_gh_release_fails() {
    let fx = fixture();
    fx.cargo_stub();
    fx.stub_stdout("curl", "CRATES_IO_JSON")
        .stub("gh", FAILING_STUB)
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("変更内容不明"));
}

#[test]
fn explain_shows_generation_failed_when_claude_errors() {
    let fx = fixture();
    fx.cargo_stub();
    fx.stub_stdout("curl", "CRATES_IO_JSON")
        .stub_stdout("gh", "GH_RELEASE_JSON")
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("GH_RELEASE_JSON", GH_RELEASE_JSON)
        .env("CLAUDE_JSON", CLAUDE_ERROR_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("要約失敗（claude 生成エラー）"));
}
