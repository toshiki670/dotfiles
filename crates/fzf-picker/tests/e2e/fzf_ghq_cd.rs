//! `fzf-ghq-cd` の E2E（実バイナリを起動して検証）。
//!
//! 検証: `--help`/`--version`、`ghq` 不在で 127、選択行の 3 列目（パス）を stdout に
//! 出力、fzf キャンセルで無出力、空 `ghq list` で候補ゼロ、リンク worktree が repo 行
//! 直下にツリー表示され（`is_main` のメインは除外）fzf へ渡る。

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;

use crate::{EMPTY_PATH, FZF_STUB, GHQ_STUB, git, path_with, write_exec};

fn fzf_ghq_cd() -> Command {
    Command::cargo_bin("fzf-ghq-cd").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    fzf_ghq_cd().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    fzf_ghq_cd()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("fzf-ghq-cd"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_ghq_exits_127() {
    fzf_ghq_cd()
        .env("PATH", EMPTY_PATH)
        .assert()
        .failure()
        .code(127)
        .stderr(predicate::str::contains("ghq command not found"));
}

/// `ghq` / `fzf` スタブと ghq root を備えたテスト環境。
struct Env {
    _root: TempDir,
    bin: TempDir,
    root_path: PathBuf,
    list_file: PathBuf,
    dump_file: PathBuf,
}

impl Env {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let bin = TempDir::new().unwrap();
        let list_file = bin.path().join("ghq-list.txt");
        let dump_file = bin.path().join("fzf-input.txt");
        fs::write(&list_file, "").unwrap();

        write_exec(bin.path(), "ghq", GHQ_STUB);
        write_exec(bin.path(), "fzf", FZF_STUB);

        let root_path = root.path().to_path_buf();
        Self {
            _root: root,
            bin,
            root_path,
            list_file,
            dump_file,
        }
    }

    /// `ghq list` が返す相対パス群を設定する。
    fn set_repos(&self, rels: &[&str]) {
        fs::write(&self.list_file, rels.join("\n")).unwrap();
    }

    /// スタブ環境を仕込んだ `fzf-ghq-cd` コマンドを得る。
    fn cmd(&self) -> Command {
        let mut cmd = fzf_ghq_cd();
        cmd.env("PATH", path_with(self.bin.path()))
            .env("GHQ_ROOT", &self.root_path)
            .env("GHQ_LIST_FILE", &self.list_file)
            .env("FZF_DUMP", &self.dump_file);
        cmd
    }

    /// fzf に渡された候補（ダンプ）を読む。
    fn candidates(&self) -> String {
        fs::read_to_string(&self.dump_file).unwrap_or_default()
    }
}

#[test]
fn selecting_an_entry_prints_its_path() {
    let env = Env::new();
    env.set_repos(&["owner/repo"]);
    let repo_path = env.root_path.join("owner/repo");
    let pick = format!("owner/repo\trepo\t{}\towner/repo", repo_path.display());

    env.cmd()
        .env("FZF_PICK", &pick)
        .assert()
        .success()
        .stdout(format!("{}\n", repo_path.display()));
}

#[test]
fn cancelling_fzf_prints_nothing_and_succeeds() {
    let env = Env::new();
    env.set_repos(&["owner/repo"]);

    // FZF_PICK 無し・終了コード 1（ESC / Ctrl-C 相当）。
    env.cmd().env("FZF_EXIT", "1").assert().success().stdout("");
}

#[test]
fn empty_repo_list_yields_no_candidates() {
    let env = Env::new(); // list_file は既定で空

    env.cmd().env("FZF_EXIT", "1").assert().success().stdout("");
    assert!(
        env.candidates().trim().is_empty(),
        "expected no candidates, got:\n{}",
        env.candidates()
    );
}

#[test]
fn linked_worktree_is_rendered_as_tree_under_repo() {
    let env = Env::new();

    // ghq root 配下に実 git repo を作り、リンク worktree を 1 本生やす。
    let repo = env.root_path.join("owner/repo");
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
    let worktree = env.root_path.join("wt-feature");
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

    env.set_repos(&["owner/repo"]);
    // 選択はせず（exit 1）、fzf に渡る候補だけを検証する。
    env.cmd().env("FZF_EXIT", "1").assert().success();

    let candidates = env.candidates();
    let repo_line = format!("owner/repo\trepo\t{}\towner/repo", repo.display());
    assert!(
        candidates.lines().any(|l| l == repo_line),
        "repo line missing:\n{candidates}"
    );
    assert!(
        candidates
            .lines()
            .any(|l| l.starts_with("└─ feature/x\tworktree\t")),
        "linked worktree tree line missing:\n{candidates}"
    );
    // メイン worktree はツリーに出さない（is_main フィルタ）。リンクは 1 本だけ。
    assert_eq!(
        candidates.matches("\tworktree\t").count(),
        1,
        "exactly one (linked) worktree tree line expected:\n{candidates}"
    );
}
