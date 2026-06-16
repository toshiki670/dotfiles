//! fzf 系ピッカー（旧 fish B 群）の共有ロジック。
//!
//! `git worktree list --porcelain` のパーサなど純ロジックを置き、各 bin
//! （`fzf-ghq-cd` ほか）から使う。旧 `__fzf_worktree_list`（awk）の Rust 版。
//!
//! この lib は **このクレート内部限定**。未公開（crates.io 非掲載）なので他の配布
//! クレートから path 依存してはいけない（release-plz git_only で cargo package が
//! 壊れる）。共有はクレート内の bin からのみ行う。

use std::path::Path;
use std::process::Command;

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

/// パース途中の 1 worktree 分の状態。
struct Pending {
    path: String,
    branch: Option<String>,
    detached: bool,
}

impl Pending {
    /// `is_main` を与えて確定した `Worktree` にする。ラベルは branch 優先、
    /// なければ detached なら `(detached)`、どちらでもなければ空文字。
    fn finish(self, is_main: bool) -> Worktree {
        let label = match (self.branch, self.detached) {
            (Some(branch), _) => branch,
            (None, true) => "(detached)".to_string(),
            (None, false) => String::new(),
        };
        Worktree {
            is_main,
            path: self.path,
            label,
        }
    }
}

/// `git worktree list --porcelain` の出力をパースする純関数。
///
/// 最初のエントリを `is_main = true`、以降のリンク worktree を `is_main = false`
/// として返す（旧 awk の `idx == 0` 判定と同じ）。
pub fn parse_worktrees(porcelain: &str) -> Vec<Worktree> {
    let mut out = Vec::new();
    let mut current: Option<Pending> = None;

    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            // 新しい worktree ブロックの開始。直前のブロックを確定する。
            if let Some(pending) = current.take() {
                let is_main = out.is_empty();
                out.push(pending.finish(is_main));
            }
            current = Some(Pending {
                path: path.to_string(),
                branch: None,
                detached: false,
            });
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/")
            && let Some(pending) = current.as_mut()
        {
            pending.branch = Some(branch.to_string());
        } else if line == "detached"
            && let Some(pending) = current.as_mut()
        {
            pending.detached = true;
        }
    }

    if let Some(pending) = current.take() {
        let is_main = out.is_empty();
        out.push(pending.finish(is_main));
    }

    out
}

/// 指定 repo の worktree 一覧を `git -C <repo> worktree list --porcelain` から取得する。
///
/// git が無い・repo でない等で失敗したら空 Vec を返す（旧 fish の `2>/dev/null` 相当）。
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

/// repo 一覧（`(ghq 相対パス, その repo の worktree 一覧)`）から fzf 候補行を組み立てる
/// 純関数。各行は `表示 \t 種別 \t パス \t ghq相対パス`。repo 行の直下に、リンク
/// worktree（`is_main == false`）を `├─` / `└─` のツリーで並べる。旧 `_fzf_ghq_cd`
/// の行組み立てロジックを IO（ghq/git 実行）から分離したもの。
pub fn picker_lines(root: &str, repos: &[(String, Vec<Worktree>)]) -> Vec<String> {
    let mut lines = Vec::new();

    for (rel, worktrees) in repos {
        let repo_path = format!("{root}/{rel}");
        // repo 行: 表示=相対パス / 種別=repo / パス=repo_path / 相対=rel
        lines.push(format!("{rel}\trepo\t{repo_path}\t{rel}"));

        let linked: Vec<&Worktree> = worktrees.iter().filter(|w| !w.is_main).collect();
        let last = linked.len().saturating_sub(1);
        for (i, w) in linked.iter().enumerate() {
            let marker = if i == last { "└─" } else { "├─" };
            // worktree 行: 表示=ツリー / 種別=worktree / パス=wt / 相対=空
            lines.push(format!("{marker} {}\tworktree\t{}\t", w.label, w.path));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_main_linked_and_detached() {
        let input = "\
worktree /repo
HEAD 0000000000000000000000000000000000000000
branch refs/heads/main

worktree /repo/.worktrees/feature
HEAD 1111111111111111111111111111111111111111
branch refs/heads/feature/foo

worktree /repo/.worktrees/wip
HEAD 2222222222222222222222222222222222222222
detached
";
        let worktrees = parse_worktrees(input);
        assert_eq!(
            worktrees,
            vec![
                Worktree {
                    is_main: true,
                    path: "/repo".to_string(),
                    label: "main".to_string(),
                },
                Worktree {
                    is_main: false,
                    path: "/repo/.worktrees/feature".to_string(),
                    label: "feature/foo".to_string(),
                },
                Worktree {
                    is_main: false,
                    path: "/repo/.worktrees/wip".to_string(),
                    label: "(detached)".to_string(),
                },
            ]
        );
    }

    #[test]
    fn empty_input_yields_empty() {
        assert!(parse_worktrees("").is_empty());
    }

    #[test]
    fn single_main_worktree() {
        let input = "worktree /only\nHEAD abc\nbranch refs/heads/main\n";
        let worktrees = parse_worktrees(input);
        assert_eq!(worktrees.len(), 1);
        assert!(worktrees[0].is_main);
        assert_eq!(worktrees[0].label, "main");
    }

    #[test]
    fn branch_label_keeps_slashes() {
        let input = "worktree /w\nbranch refs/heads/feature/deep/name\n";
        let worktrees = parse_worktrees(input);
        assert_eq!(worktrees[0].label, "feature/deep/name");
    }

    #[test]
    fn missing_branch_and_not_detached_yields_empty_label() {
        let input = "worktree /w\nHEAD abc\n";
        let worktrees = parse_worktrees(input);
        assert_eq!(worktrees[0].label, "");
    }

    fn main_wt(path: &str) -> Worktree {
        Worktree {
            is_main: true,
            path: path.to_string(),
            label: "main".to_string(),
        }
    }

    fn linked_wt(path: &str, label: &str) -> Worktree {
        Worktree {
            is_main: false,
            path: path.to_string(),
            label: label.to_string(),
        }
    }

    #[test]
    fn picker_lines_empty_repos() {
        assert!(picker_lines("/root", &[]).is_empty());
    }

    #[test]
    fn picker_lines_repo_without_linked_worktrees() {
        let repos = vec![("owner/repo".to_string(), vec![main_wt("/root/owner/repo")])];
        assert_eq!(
            picker_lines("/root", &repos),
            vec!["owner/repo\trepo\t/root/owner/repo\towner/repo".to_string()]
        );
    }

    #[test]
    fn picker_lines_filters_main_and_marks_tree() {
        let repos = vec![(
            "o/r".to_string(),
            vec![
                main_wt("/root/o/r"),
                linked_wt("/wt/a", "feat/a"),
                linked_wt("/wt/b", "(detached)"),
            ],
        )];
        assert_eq!(
            picker_lines("/root", &repos),
            vec![
                "o/r\trepo\t/root/o/r\to/r".to_string(),
                "├─ feat/a\tworktree\t/wt/a\t".to_string(),
                "└─ (detached)\tworktree\t/wt/b\t".to_string(),
            ]
        );
    }

    #[test]
    fn picker_lines_single_linked_uses_last_marker() {
        let repos = vec![(
            "o/r".to_string(),
            vec![main_wt("/root/o/r"), linked_wt("/wt/only", "feat")],
        )];
        let lines = picker_lines("/root", &repos);
        assert_eq!(lines[1], "└─ feat\tworktree\t/wt/only\t");
    }

    #[test]
    fn picker_lines_multiple_repos_are_independent() {
        let repos = vec![
            ("a/x".to_string(), vec![main_wt("/root/a/x")]),
            (
                "b/y".to_string(),
                vec![main_wt("/root/b/y"), linked_wt("/wt/y1", "topic")],
            ),
        ];
        assert_eq!(
            picker_lines("/root", &repos),
            vec![
                "a/x\trepo\t/root/a/x\ta/x".to_string(),
                "b/y\trepo\t/root/b/y\tb/y".to_string(),
                "└─ topic\tworktree\t/wt/y1\t".to_string(),
            ]
        );
    }
}
