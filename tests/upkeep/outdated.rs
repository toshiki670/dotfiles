//! `outdated` の E2E（実バイナリ + スタブ PM/外部コマンドで検証）。
//!
//! 外部コマンド（brew/mise/cargo/curl/gh/claude）は環境変数の中身をそのまま stdout に
//! 返すスタブ（[`stdout_stub_body`]）で差し替える。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{
    EMPTY_PATH, dispatch_stub_body, recording_stdout_stub_body, stdout_stub_body, stub_body,
    write_exec,
};

const BREW_JSON: &str = r#"{"formulae":[{"name":"bat","installed_versions":["0.24.0"],"current_version":"0.25.0","pinned":false,"pinned_version":null}],"casks":[]}"#;
const MISE_JSON: &str = r#"{"jq":{"name":"jq","requested":"1.6","current":"1.6","bump":"1.8","latest":"1.8.2","source":{"type":"mise.toml","path":"/tmp/mise.toml"}}}"#;
const BREW_INFO_JSON: &str = r#"{"formulae":[{"homepage":"https://github.com/sharkdp/bat","urls":{"stable":{"url":"https://github.com/sharkdp/bat/archive/refs/tags/v0.26.1.tar.gz"}}}],"casks":[]}"#;
const BREW_INFO_NON_GITHUB_JSON: &str = r#"{"formulae":[{"homepage":"https://ffmpeg.org/","urls":{"stable":{"url":"https://ffmpeg.org/releases/ffmpeg-8.0.tar.xz"}}}],"casks":[]}"#;
const MISE_TOOL_AQUA_JSON: &str = r#"{"backend":"aqua:jqlang/jq"}"#;
const MISE_TOOL_PLUGIN_JSON: &str = r#"{"backend":"asdf:mise-plugins/asdf-jq"}"#;
const CARGO_TABLE: &str =
    "Package      Installed  Latest   Needs update\ncargo-audit  v0.17.0    v0.18.0  Yes";
const CRATES_IO_JSON: &str = r#"{"crate":{"repository":"https://github.com/rustsec/rustsec"}}"#;
/// `gh api repos/rustsec/rustsec/releases`（cargo-audit: v0.17.0 -> v0.18.0）。
const GH_RELEASES_RUSTSEC: &str = r#"[
{"tag_name":"v0.18.0","body":"What's Changed\n\n* Fix bug X","html_url":"https://github.com/rustsec/rustsec/releases/tag/v0.18.0","draft":false,"prerelease":false},
{"tag_name":"v0.17.0","body":"older","html_url":"https://github.com/rustsec/rustsec/releases/tag/v0.17.0","draft":false,"prerelease":false}
]"#;

/// `gh api repos/sharkdp/bat/releases`（bat: 0.24.0 -> 0.25.0）。
const GH_RELEASES_BAT: &str = r#"[
{"tag_name":"v0.25.0","body":"What's Changed\n\n* Fix bug X","html_url":"https://github.com/sharkdp/bat/releases/tag/v0.25.0","draft":false,"prerelease":false},
{"tag_name":"v0.24.0","body":"older","html_url":"https://github.com/sharkdp/bat/releases/tag/v0.24.0","draft":false,"prerelease":false}
]"#;

/// `gh api repos/sharkdp/bat/releases` から `current`（0.24.0）が落ちたもの。
const GH_RELEASES_BAT_WITHOUT_CURRENT: &str = r#"[
{"tag_name":"v0.25.0","body":"What's Changed","html_url":"https://github.com/sharkdp/bat/releases/tag/v0.25.0","draft":false,"prerelease":false},
{"tag_name":"v0.23.0","body":"much older","html_url":"https://github.com/sharkdp/bat/releases/tag/v0.23.0","draft":false,"prerelease":false}
]"#;

/// `gh api repos/jqlang/jq/releases`（jq: 1.6 -> 1.8.2）。タグはパッケージ名接頭辞で、
/// 範囲に正式リリース3件と prerelease 1件が挟まる。
const GH_RELEASES_JQ: &str = r#"[
{"tag_name":"jq-1.8.2","body":"jq 1.8.2 fixes the parser","html_url":"https://github.com/jqlang/jq/releases/tag/jq-1.8.2","draft":false,"prerelease":false},
{"tag_name":"jq-1.8.2-rc1","body":"release candidate","html_url":"https://github.com/jqlang/jq/releases/tag/jq-1.8.2-rc1","draft":false,"prerelease":true},
{"tag_name":"jq-1.8.1","body":"jq 1.8.1 adds a builtin","html_url":"https://github.com/jqlang/jq/releases/tag/jq-1.8.1","draft":false,"prerelease":false},
{"tag_name":"jq-1.7.1","body":"jq 1.7.1 drops an option","html_url":"https://github.com/jqlang/jq/releases/tag/jq-1.7.1","draft":false,"prerelease":false},
{"tag_name":"jq-1.6","body":"jq 1.6","html_url":"https://github.com/jqlang/jq/releases/tag/jq-1.6","draft":false,"prerelease":false}
]"#;
const CLAUDE_SUMMARY_JSON: &str = r#"{"type":"result","is_error":false,"structured_output":{"release_notes_ja":"新機能Xを追加"}}"#;
const CLAUDE_ERROR_JSON: &str = r#"{"is_error":true,"errors":["boom"]}"#;
const FAILING_STUB: &str = "#!/bin/sh\nexit 1\n";

/// `sleep` を使うスタブ本体を組み立てる。テストの PATH はスタブディレクトリだけに絞って
/// あるので、スタブ自身が `sleep` を引ける PATH をここで補う。
fn sleeping_stub_body(body: &str) -> String {
    format!("#!/bin/sh\nPATH=/usr/bin:/bin\nexport PATH\ncat >/dev/null\n{body}")
}

/// リポジトリごとに応答を振り分ける `case` 本体。取得を遅らせる等の細工を前に挟むために
/// スタブ本体とは分けてある。
const GH_BY_REPO: &str = concat!(
    "case \"$*\" in\n",
    "  *sharkdp/bat*) printf '%s\\n' \"$GH_RELEASES_BAT\" ;;\n",
    "  *) printf '%s\\n' \"$GH_RELEASES_JQ\" ;;\n",
    "esac\n"
);

/// リポジトリごとに応答を振り分ける `gh`。
fn gh_by_repo_stub() -> String {
    format!("#!/bin/sh\ncat >/dev/null\n{GH_BY_REPO}")
}

/// brew 側（`sharkdp/bat`）のリリース取得だけを遅らせる `gh`。
fn slow_for_brew_gh_stub() -> String {
    sleeping_stub_body(&format!(
        "case \"$*\" in\n  *sharkdp/bat*) sleep 0.5 ;;\nesac\n{GH_BY_REPO}"
    ))
}

/// 呼び出しの開始と終了を `$UPKEEP_LOG` に刻む `gh`。区間が重なるかで並行性を見る。
fn overlap_probe_gh_stub() -> String {
    sleeping_stub_body(&format!(
        "printf 'start\\n' >> \"$UPKEEP_LOG\"\nsleep 0.5\nprintf 'end\\n' >> \"$UPKEEP_LOG\"\n{GH_BY_REPO}"
    ))
}

fn outdated() -> Command {
    let mut cmd = Command::cargo_bin("upkeep").unwrap();
    cmd.arg("outdated");
    cmd
}

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

impl Fixture {
    /// `name` を、環境変数 `env_var` の中身をそのまま返すスタブとして置く。
    fn stub_stdout(&self, name: &str, env_var: &str) -> &Self {
        write_exec(&self.bin, name, &stdout_stub_body(env_var));
        self
    }

    /// `name` を、呼び出し引数を `$UPKEEP_LOG` に残す [`Fixture::stub_stdout`] として置く。
    fn stub_stdout_recording(&self, name: &str, env_var: &str) -> &Self {
        write_exec(&self.bin, name, &recording_stdout_stub_body(name, env_var));
        self
    }

    /// `name` を、サブコマンドで返す環境変数を切り替えるスタブとして置く。
    fn stub_dispatch(&self, name: &str, cases: &[(&str, &str)]) -> &Self {
        write_exec(&self.bin, name, &dispatch_stub_body(cases));
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
    fx.stub_dispatch("brew", &[("outdated", "BREW_JSON")]);

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
    fx.stub_dispatch("mise", &[("outdated", "MISE_JSON")]);

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
    fx.stub_dispatch("brew", &[("outdated", "BREW_JSON")])
        .stub_dispatch("mise", &[("outdated", "MISE_JSON")])
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
    fx.stub_dispatch("brew", &[("outdated", "BREW_JSON")]);
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
        .stub_stdout("gh", "GH_RELEASES_JSON")
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_RUSTSEC)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("要約: 新機能Xを追加"))
        .stdout(predicate::str::contains(
            "出典 1件: https://github.com/rustsec/rustsec/releases/tag/v0.18.0",
        ));
}

/// 要約の呼び出しは、呼び出し元の設定とツールを持ち込まない形で行う。持ち込むと要約では
/// なく「要約して回答した」という行為の報告が返るため、この引数自体が修正の本体になる。
#[test]
fn explain_isolates_the_claude_invocation() {
    let fx = fixture();
    fx.cargo_stub();
    fx.stub_stdout("curl", "CRATES_IO_JSON")
        .stub_stdout("gh", "GH_RELEASES_JSON")
        .stub_stdout_recording("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_RUSTSEC)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success();

    let log = fs::read_to_string(&fx.log).unwrap_or_default();
    for expected in ["--safe-mode", "--tools", "release_notes_ja"] {
        assert!(log.contains(expected), "missing {expected}:\n{log}");
    }
}

/// 解説付きは1件が複数行になるので、パッケージ同士は空行で区切る。
#[test]
fn explain_separates_packages_with_a_blank_line() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub("gh", &gh_by_repo_stub())
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_JSON)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("GH_RELEASES_BAT", GH_RELEASES_BAT)
        .env("GH_RELEASES_JQ", GH_RELEASES_JQ)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("\n\n[mise] jq"));
}

/// 解説なしの一覧は1行ずつなので、空行で間延びさせない。
#[test]
fn list_without_explain_has_no_blank_lines_between_packages() {
    let fx = fixture();
    fx.stub_dispatch("brew", &[("outdated", "BREW_JSON")])
        .stub_dispatch("mise", &[("outdated", "MISE_JSON")]);

    outdated()
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("MISE_JSON", MISE_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[brew] bat: 0.24.0 -> 0.25.0\n[mise] jq: 1.6 -> 1.8.2",
        ));
}

#[test]
fn explain_summarizes_brew_package() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_stdout("gh", "GH_RELEASES_JSON")
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_BAT)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("[brew] bat: 0.24.0 -> 0.25.0"))
        .stdout(predicate::str::contains("要約: 新機能Xを追加"));
}

#[test]
fn explain_summarizes_mise_tool_via_backend() {
    let fx = fixture();
    fx.stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub_stdout("gh", "GH_RELEASES_JSON")
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_JQ)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("[mise] jq: 1.6 -> 1.8.2"))
        .stdout(predicate::str::contains("要約: 新機能Xを追加"));
}

/// `current` の次から `latest` までを全部要約に含め、出典としてそれぞれを挙げる。範囲に
/// 挟まる prerelease は含めない。
#[test]
fn explain_covers_every_release_between_current_and_latest() {
    let fx = fixture();
    fx.stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub_stdout("gh", "GH_RELEASES_JSON")
    .stub_stdout("claude", "CLAUDE_JSON");

    let assert = outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_JQ)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    // URL は行末で終わるので、`jq-1.8.2` が `jq-1.8.2-rc1` に前方一致するのを改行で切る。
    for tag in ["jq-1.8.2", "jq-1.8.1", "jq-1.7.1"] {
        let url = format!("https://github.com/jqlang/jq/releases/tag/{tag}\n");
        assert!(stdout.contains(&url), "missing source {tag}:\n{stdout}");
    }
    assert!(
        !stdout.contains("tag/jq-1.6\n"),
        "current itself was summarized:\n{stdout}"
    );
    assert!(
        !stdout.contains("rc1"),
        "prerelease was summarized:\n{stdout}"
    );
    assert!(!stdout.contains('※'), "unexpected range note:\n{stdout}");
}

/// `current` のタグを引けないときは、最新1件しか見ていないことを明示する（1件だけの
/// 要約が「1件しか無かった」のか「起点を見失った」のかを読み分けられるように）。
#[test]
fn explain_marks_the_range_when_current_tag_is_unknown() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_stdout("gh", "GH_RELEASES_JSON")
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_BAT_WITHOUT_CURRENT)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "※ 0.24.0 以降の範囲を特定できず、最新1件のみ要約",
        ));
}

#[test]
fn explain_shows_homepage_when_upstream_is_not_github() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_NON_GITHUB_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("変更内容不明"))
        .stdout(predicate::str::contains("参考: https://ffmpeg.org/"));
}

#[test]
fn explain_shows_bare_unavailable_when_backend_is_out_of_scope() {
    let fx = fixture();
    // asdf backend が名指すのはプラグインであってツール本体ではないので解決しない。
    fx.stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_PLUGIN_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("変更内容不明"))
        .stdout(predicate::str::contains("参考:").not());
}

#[test]
fn explain_shows_repo_page_when_release_is_missing() {
    let fx = fixture();
    fx.stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub("gh", FAILING_STUB)
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("変更内容不明"))
        .stdout(predicate::str::contains(
            "参考: https://github.com/jqlang/jq",
        ));
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
        .stub_stdout("gh", "GH_RELEASES_JSON")
        .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("CARGO_TABLE", CARGO_TABLE)
        .env("CRATES_IO_JSON", CRATES_IO_JSON)
        .env("GH_RELEASES_JSON", GH_RELEASES_RUSTSEC)
        .env("CLAUDE_JSON", CLAUDE_ERROR_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("要約失敗: boom"));
}

/// 並行に解決すると完了順はばらつく。表示は検出順（brew → mise）に固定する。
#[test]
fn explain_keeps_detection_order_when_resolution_finishes_out_of_order() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    // 先に検出される brew のリリース取得だけを遅らせ、mise を先に完了させる。
    .stub("gh", &slow_for_brew_gh_stub())
    .stub_stdout("claude", "CLAUDE_JSON");

    let assert = outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_JSON)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("GH_RELEASES_BAT", GH_RELEASES_BAT)
        .env("GH_RELEASES_JQ", GH_RELEASES_JQ)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    let brew = stdout.find("[brew] bat").expect("brew line missing");
    let mise = stdout.find("[mise] jq").expect("mise line missing");
    assert!(brew < mise, "detection order not preserved:\n{stdout}");
}

/// パッケージ間に依存は無いので、解決は重ねて走らせる。
#[test]
fn explain_resolves_packages_concurrently() {
    let fx = fixture();
    fx.stub_dispatch(
        "brew",
        &[("outdated", "BREW_JSON"), ("info", "BREW_INFO_JSON")],
    )
    .stub_dispatch(
        "mise",
        &[("outdated", "MISE_JSON"), ("tool", "MISE_TOOL_JSON")],
    )
    .stub("gh", &overlap_probe_gh_stub())
    .stub_stdout("claude", "CLAUDE_JSON");

    outdated()
        .arg("--explain")
        .env("PATH", &fx.bin)
        .env("UPKEEP_LOG", &fx.log)
        .env("BREW_JSON", BREW_JSON)
        .env("BREW_INFO_JSON", BREW_INFO_JSON)
        .env("MISE_JSON", MISE_JSON)
        .env("MISE_TOOL_JSON", MISE_TOOL_AQUA_JSON)
        .env("GH_RELEASES_BAT", GH_RELEASES_BAT)
        .env("GH_RELEASES_JQ", GH_RELEASES_JQ)
        .env("CLAUDE_JSON", CLAUDE_SUMMARY_JSON)
        .assert()
        .success();

    // 直列なら start,end,start,end になる。重なっていれば両方の start が先に並ぶ。
    let log = fs::read_to_string(&fx.log).unwrap_or_default();
    let events: Vec<&str> = log.lines().collect();
    assert_eq!(
        events,
        ["start", "start", "end", "end"],
        "gh calls did not overlap:\n{log}"
    );
}
