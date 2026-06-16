//! `fzf-worktree-remove` の E2E（実バイナリ + 実 git worktree で検証）。
//!
//! 検証: `--help`/`--version`、非 git repo で失敗、削除候補なしで `No worktrees to
//! delete`、確認 `y` で worktree 削除、確認 `n` で残置、fzf キャンセルで残置、削除対象
//! の内側から実行したときに退避先（メイン worktree）パスを stdout 出力。

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, FZF_STUB, git, path_with, write_exec};

fn fzf_worktree_remove() -> Command {
    Command::cargo_bin("fzf-worktree-remove").unwrap()
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn meta_flags_succeed(#[case] flag: &str) {
    fzf_worktree_remove().arg(flag).assert().success();
}

#[test]
fn outside_git_repo_fails() {
    let dir = TempDir::new().unwrap();
    fzf_worktree_remove()
        .current_dir(dir.path())
        .env("PATH", EMPTY_PATH) // git も見えない環境
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a git repository"));
}

/// リンク worktree を 1 本持つ実 git repo と fzf スタブを用意する。
struct RemoveFixture {
    _root: TempDir,
    repo: PathBuf,
    worktree: PathBuf,
    bin: PathBuf,
    dump: PathBuf,
}

fn remove_fixture() -> RemoveFixture {
    let root = TempDir::new().unwrap();
    let repo = root.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(
        &repo,
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
    );
    let worktree = root.path().join("wt-feature");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/x",
            worktree.to_str().unwrap(),
        ],
    );

    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let dump = bin.join("fzf-input.txt");
    write_exec(&bin, "fzf", FZF_STUB);
    RemoveFixture {
        _root: root,
        repo,
        worktree,
        bin,
        dump,
    }
}

#[test]
fn reports_when_no_linked_worktrees() {
    // メインのみの repo（リンク worktree を消しておく）。
    let fx = remove_fixture();
    git(
        &fx.repo,
        &["worktree", "remove", fx.worktree.to_str().unwrap()],
    );

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("No worktrees to delete"));
}

#[test]
fn confirm_yes_deletes_worktree() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    fzf_worktree_remove()
        .current_dir(&fx.repo) // メイン側（対象の内側ではない）→ cd 不要
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout("") // 内側でないので退避パスは出さない
        .stderr(predicate::str::contains("削除しました"));

    assert!(!fx.worktree.exists(), "worktree dir should be gone");
}

#[test]
fn confirm_no_keeps_worktree() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout("");

    assert!(fx.worktree.exists(), "worktree dir should remain");
}

#[test]
fn cancel_fzf_keeps_worktree() {
    let fx = remove_fixture();

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_EXIT", "1") // ESC 相当
        .assert()
        .success()
        .stdout("");

    assert!(fx.worktree.exists(), "worktree dir should remain");
    // 候補にはメインを含めず、リンク 1 本だけ並ぶ。
    let candidates = fs::read_to_string(&fx.dump).unwrap_or_default();
    assert!(
        candidates.lines().any(|l| l.starts_with("feature/x\t")),
        "linked worktree candidate missing:\n{candidates}"
    );
    assert_eq!(
        candidates.lines().count(),
        1,
        "only the linked worktree expected"
    );
}

#[test]
fn from_inside_target_prints_cd_path() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    let assert = fzf_worktree_remove()
        .current_dir(&fx.worktree) // 削除対象の内側にいる
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("y\n")
        .assert()
        .success();

    // 退避先（メイン worktree）パスが stdout に出る。
    let out = assert.get_output();
    let printed = String::from_utf8_lossy(&out.stdout);
    let printed = printed.trim();
    assert!(!printed.is_empty(), "expected a cd target path on stdout");
    assert_eq!(
        Path::new(printed).canonicalize().ok(),
        fx.repo.canonicalize().ok(),
        "cd target should be the main worktree"
    );
    assert!(!fx.worktree.exists(), "worktree dir should be gone");
}
