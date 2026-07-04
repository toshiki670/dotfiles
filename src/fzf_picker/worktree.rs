//! git worktree ドメイン: worktree を表す型と、その一覧を取得する IO。
//!
//! porcelain 文字列の**パースは [`super::parse`]** に分離してある（この層は「git を
//! 叩いて worktree 一覧を得る」責務だけを持つ）。

use std::path::Path;
use std::process::Command;

use super::parse::parse_worktrees;

/// `git worktree list --porcelain` の 1 worktree 分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    /// メイン worktree（porcelain の最初のエントリ）なら true。
    pub is_main: bool,
    /// worktree の絶対パス。
    pub path: String,
    /// 表示用ラベル: branch 名、detached なら `(detached)`、いずれでもなければ空。
    pub label: String,
}

/// 指定 repo の worktree 一覧を `git -C <repo> worktree list --porcelain` から取得する。
///
/// git が無い・repo でない等で失敗したら空 Vec を返す。
/// 取得した porcelain は [`parse_worktrees`] でパースする。
pub fn list_worktrees(repo: &Path) -> Vec<Worktree> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "list", "--porcelain"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            parse_worktrees(&String::from_utf8_lossy(&output.stdout))
        }
        _ => Vec::new(),
    }
}
