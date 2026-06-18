//! `fzf-gh` の E2E（実バイナリ + gh/fzf スタブで検証）。
//!
//! 検証: gh 不在で失敗、Issue 選択→アクション選択で `gh issue edit 341` 出力、
//! PR 選択で `gh pr checkout 7` 出力、項目段キャンセルで無出力、アクション段キャンセルで
//! 無出力。`gh` は第1引数で issue/pr を出し分け、あらかじめ `表示\tkind\t番号` に整形した
//! canned TSV を返すスタブ（`--jq` はバイパス）。`fzf` は stdin にタブを含むかで
//! 項目段/アクション段を判別する 2 段対応スタブ。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::{EMPTY_PATH, path_with, write_exec};

/// `gh` スタブ: `issue`/`pr` を第1引数で判定し、canned TSV をファイルから返す。
const GH_STUB: &str = "#!/bin/sh\ncase \"$1\" in\n  issue) cat \"${GH_ISSUE_FILE:-/dev/null}\" ;;\n  pr) cat \"${GH_PR_FILE:-/dev/null}\" ;;\nesac\n";

/// `fzf` スタブ（2段対応）: stdin にタブを含めば項目段
/// （`$FZF_PICK_ITEM` / `$FZF_EXIT_ITEM`）、含まなければアクション段
/// （`$FZF_PICK_ACTION` / `$FZF_EXIT_ACTION`）として振る舞う。終了コードが 0 のときだけ
/// 選択行を出力する（非 0 = fzf キャンセル相当）。
const FZF_STUB: &str = "#!/bin/sh\ninput=$(cat)\nif printf '%s' \"$input\" | grep -q \"$(printf '\\t')\"; then\n  sel=\"$FZF_PICK_ITEM\"; code=\"${FZF_EXIT_ITEM:-0}\"\nelse\n  sel=\"$FZF_PICK_ACTION\"; code=\"${FZF_EXIT_ACTION:-0}\"\nfi\n[ \"$code\" = \"0\" ] && [ -n \"$sel\" ] && printf '%s\\n' \"$sel\"\nexit \"$code\"\n";

const ISSUE_LINE: &str = "[issue] #341 demo @me\tissue\t341";
const PR_LINE: &str = "[pr] #7 fix @you\tpr\t7";

fn fzf_gh() -> Command {
    Command::cargo_bin("fzf-gh").unwrap()
}

struct GhFixture {
    _root: TempDir,
    bin: PathBuf,
    issue_file: PathBuf,
    pr_file: PathBuf,
}

fn fixture() -> GhFixture {
    let root = TempDir::new().unwrap();
    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    write_exec(&bin, "gh", GH_STUB);
    write_exec(&bin, "fzf", FZF_STUB);

    let issue_file = root.path().join("issues.tsv");
    let pr_file = root.path().join("prs.tsv");
    fs::write(&issue_file, format!("{ISSUE_LINE}\n")).unwrap();
    fs::write(&pr_file, format!("{PR_LINE}\n")).unwrap();

    GhFixture {
        _root: root,
        bin,
        issue_file,
        pr_file,
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
    let fx = fixture();
    fzf_gh()
        .env("PATH", path_with(&fx.bin))
        .env("GH_ISSUE_FILE", &fx.issue_file)
        .env("GH_PR_FILE", &fx.pr_file)
        .env("FZF_PICK_ITEM", ISSUE_LINE)
        .env("FZF_PICK_ACTION", "edit")
        .assert()
        .success()
        .stdout("gh issue edit 341\n");
}

#[test]
fn pr_selection_builds_checkout_command() {
    let fx = fixture();
    fzf_gh()
        .env("PATH", path_with(&fx.bin))
        .env("GH_ISSUE_FILE", &fx.issue_file)
        .env("GH_PR_FILE", &fx.pr_file)
        .env("FZF_PICK_ITEM", PR_LINE)
        .env("FZF_PICK_ACTION", "checkout")
        .assert()
        .success()
        .stdout("gh pr checkout 7\n");
}

#[test]
fn cancel_item_stage_no_output() {
    let fx = fixture();
    fzf_gh()
        .env("PATH", path_with(&fx.bin))
        .env("GH_ISSUE_FILE", &fx.issue_file)
        .env("GH_PR_FILE", &fx.pr_file)
        .env("FZF_EXIT_ITEM", "1")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn cancel_action_stage_no_output() {
    let fx = fixture();
    fzf_gh()
        .env("PATH", path_with(&fx.bin))
        .env("GH_ISSUE_FILE", &fx.issue_file)
        .env("GH_PR_FILE", &fx.pr_file)
        .env("FZF_PICK_ITEM", ISSUE_LINE)
        .env("FZF_EXIT_ACTION", "1")
        .assert()
        .success()
        .stdout("");
}
