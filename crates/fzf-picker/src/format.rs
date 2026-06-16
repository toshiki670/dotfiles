//! 行成形: [`Worktree`] 等の構造から fzf 候補行（タブ区切り）を組み立てる純ロジック。
//!
//! - [`picker_lines`]: ghq repo + リンク worktree のツリー（`fzf-ghq-cd` 用）
//! - [`removal_lines`]: 削除対象のリンク worktree 一覧（`fzf-worktree-remove` 用）

use crate::worktree::Worktree;

/// repo 一覧（`(ghq 相対パス, その repo の worktree 一覧)`）から fzf 候補行を組み立てる。
/// 各行は `表示 \t 種別 \t パス \t ghq相対パス`。repo 行の直下に、リンク worktree
/// （`is_main == false`）を `├─` / `└─` のツリーで並べる。
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

/// リンク worktree（`is_main == false`）を `表示 \t パス` の 2 列で返す。メイン worktree
/// は削除対象外なので除外する。
pub fn removal_lines(worktrees: &[Worktree]) -> Vec<String> {
    worktrees
        .iter()
        .filter(|w| !w.is_main)
        .map(|w| format!("{}\t{}", w.label, w.path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            picker_lines("/root", &repos)[1],
            "└─ feat\tworktree\t/wt/only\t"
        );
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

    #[test]
    fn removal_lines_excludes_main_and_uses_two_columns() {
        let worktrees = vec![
            main_wt("/root/o/r"),
            linked_wt("/wt/a", "feat/a"),
            linked_wt("/wt/b", "(detached)"),
        ];
        assert_eq!(
            removal_lines(&worktrees),
            vec!["feat/a\t/wt/a".to_string(), "(detached)\t/wt/b".to_string()]
        );
    }

    #[test]
    fn removal_lines_empty_when_only_main() {
        assert!(removal_lines(&[main_wt("/root/o/r")]).is_empty());
    }
}
