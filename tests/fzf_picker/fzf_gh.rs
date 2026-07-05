//! `fzf-gh` の E2E（実バイナリ + gh/fzf スタブ + 実 jq で検証）。
//!
//! 検証: gh 不在で失敗、Issue 選択→アクション選択で `gh issue edit 341` 出力、
//! PR 選択で `gh pr checkout 7` 出力、項目段キャンセルで無出力、アクション段キャンセルで
//! 無出力、そして **title のタブ/改行が候補表示でサニタイズされる**こと（列ずれ・行割れの
//! デグレ防止）。
//!
//! `gh` スタブは bin が渡す `--jq` 式を **実 jq** で canned JSON に適用する（`gh` の
//! `--jq` と等価）。これにより `jq_for` のサニタイズ（`gsub`）まで通しで効いているかを
//! 検証できる。`fzf` スタブは stdin にタブを含むかで項目段/アクション段を判別し、項目段は
//! `$FZF_PICK_NTH` 行目の **実際の候補行** を選ぶ（表示フォーマットにハードコードで依存
//! しない）。`git`/`jq` は実 PATH に残るので実物を使い、`gh`/`fzf` だけ差し替える。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::{EMPTY_PATH, path_with, write_exec};

/// `gh` スタブ: 第1引数（issue/pr）に応じ、bin が渡した `--jq` 式を実 jq で canned JSON に
/// 適用する（`gh issue list … --jq` と等価）。
const GH_STUB: &str = "#!/bin/sh\ngroup=\"$1\"\nprog=\"\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--jq\" ]; then prog=\"$2\"; fi\n  shift\ndone\ncase \"$group\" in\n  issue) jq -r \"$prog\" \"${GH_ISSUE_JSON:-/dev/null}\" ;;\n  pr) jq -r \"$prog\" \"${GH_PR_JSON:-/dev/null}\" ;;\nesac\n";

/// `fzf` スタブ（2段対応）: stdin にタブを含めば項目段、含まなければアクション段。
/// 項目段は `$FZF_DUMP` に候補を保存し、`$FZF_PICK_NTH`（既定 1）行目の実候補行を選ぶ。
/// アクション段は `$FZF_PICK_ACTION` を返す。終了コードが 0 のときだけ選択行を出す
/// （`$FZF_EXIT_ITEM` / `$FZF_EXIT_ACTION` で各段のキャンセルを再現）。
const FZF_STUB: &str = "#!/bin/sh\ninput=$(cat)\nif printf '%s' \"$input\" | grep -q \"$(printf '\\t')\"; then\n  [ -n \"$FZF_DUMP\" ] && printf '%s\\n' \"$input\" > \"$FZF_DUMP\"\n  code=\"${FZF_EXIT_ITEM:-0}\"\n  sel=$(printf '%s\\n' \"$input\" | sed -n \"${FZF_PICK_NTH:-1}p\")\nelse\n  code=\"${FZF_EXIT_ACTION:-0}\"\n  sel=\"$FZF_PICK_ACTION\"\nfi\n[ \"$code\" = \"0\" ] && [ -n \"$sel\" ] && printf '%s\\n' \"$sel\"\nexit \"$code\"\n";

const NORMAL_ISSUE: &str =
    r#"[{"number":341,"title":"demo issue","author":{"login":"me"},"assignees":[]}]"#;
const NORMAL_PR: &str =
    r#"[{"number":7,"title":"fix bug","author":{"login":"you"},"assignees":[]}]"#;
const EMPTY_JSON: &str = "[]";

fn fzf_gh() -> Command {
    let mut cmd = Command::cargo_bin("fzf-picker").unwrap();
    cmd.arg("fzf-gh");
    cmd
}

struct GhFixture {
    _root: TempDir,
    bin: PathBuf,
    issue_json: PathBuf,
    pr_json: PathBuf,
    dump: PathBuf,
}

/// gh/fzf スタブを置き、issue/pr の canned JSON を書き出した一時環境を用意する。
fn setup(issue_json: &str, pr_json: &str) -> GhFixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    write_exec(&bin, "gh", GH_STUB);
    write_exec(&bin, "fzf", FZF_STUB);

    let issue = root.path().join("issues.json");
    let pr = root.path().join("prs.json");
    fs::write(&issue, issue_json).unwrap();
    fs::write(&pr, pr_json).unwrap();
    let dump = root.path().join("fzf-candidates.txt");

    GhFixture {
        _root: root,
        bin,
        issue_json: issue,
        pr_json: pr,
        dump,
    }
}

impl GhFixture {
    /// スタブを効かせた `fzf-gh` コマンド（PATH 差し替え + JSON/ダンプ env）を組む。
    fn cmd(&self) -> Command {
        let mut c = fzf_gh();
        c.env("PATH", path_with(&self.bin))
            .env("GH_ISSUE_JSON", &self.issue_json)
            .env("GH_PR_JSON", &self.pr_json)
            .env("FZF_DUMP", &self.dump);
        c
    }
}

#[test]
fn gh_missing_fails() {
    fzf_gh()
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .stderr(predicate::str::contains("gh not found"));
}

#[test]
fn issue_selection_builds_edit_command() {
    let fx = setup(NORMAL_ISSUE, NORMAL_PR);
    fx.cmd()
        .env("FZF_PICK_NTH", "1") // 1 件目 = Issue #341
        .env("FZF_PICK_ACTION", "edit")
        .assert()
        .success()
        .stdout("gh issue edit 341\n");
}

#[test]
fn pr_selection_builds_checkout_command() {
    let fx = setup(NORMAL_ISSUE, NORMAL_PR);
    fx.cmd()
        .env("FZF_PICK_NTH", "2") // 2 件目 = PR #7
        .env("FZF_PICK_ACTION", "checkout")
        .assert()
        .success()
        .stdout("gh pr checkout 7\n");
}

#[test]
fn cancel_item_stage_no_output() {
    let fx = setup(NORMAL_ISSUE, NORMAL_PR);
    fx.cmd()
        .env("FZF_EXIT_ITEM", "1")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn cancel_action_stage_no_output() {
    let fx = setup(NORMAL_ISSUE, NORMAL_PR);
    fx.cmd()
        .env("FZF_PICK_NTH", "1")
        .env("FZF_EXIT_ACTION", "1")
        .assert()
        .success()
        .stdout("");
}

/// レビュー指摘の回帰テスト: title にタブ・改行が含まれても、候補の表示列が空白へ
/// サニタイズされ（`jq_for` の `gsub`）、機械列（kind/番号）がずれないこと。`jq` の
/// サニタイズを外すとこの候補行が崩れて落ちる。
#[test]
fn title_with_tab_and_newline_is_sanitized() {
    let dirty_issue = r#"[{"number":341,"title":"foo\tbar\nbaz qux","author":{"login":"me"},"assignees":[{"login":"a"}]}]"#;
    let fx = setup(dirty_issue, EMPTY_JSON);

    // 最終コマンドは正しく組める（末尾読みの保険込み）。
    fx.cmd()
        .env("FZF_PICK_NTH", "1")
        .env("FZF_PICK_ACTION", "edit")
        .assert()
        .success()
        .stdout("gh issue edit 341\n");

    // fzf に渡った候補行は、表示列にタブ・改行を含まない 3 列ちょうど。
    let dumped = fs::read_to_string(&fx.dump).unwrap();
    let line = dumped.trim_end_matches('\n');
    assert_eq!(line, "[issue] #341 foo bar baz qux @me →a\tissue\t341");
    assert_eq!(
        line.split('\t').count(),
        3,
        "display column must stay tab-free (kind/number は末尾2列のまま): {line:?}"
    );
}
