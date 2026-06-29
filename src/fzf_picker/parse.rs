//! パース: テキストを構造へ変換する純ロジック（IO なし）。
//!
//! - [`parse_worktrees`]: `git worktree list --porcelain` → [`Worktree`] 列
//! - [`parse_abbr_path`]: cdabbr の省略パス → ベース種別 + セグメント列

use super::worktree::Worktree;

/// パース途中の 1 worktree 分の状態。
struct Pending {
    path: String,
    branch: Option<String>,
    detached: bool,
}

impl Pending {
    /// `is_main` を与えて確定した [`Worktree`] にする。ラベルは branch 優先、なければ
    /// detached なら `(detached)`、どちらでもなければ空文字。
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
/// 最初のエントリを `is_main = true`、以降のリンク worktree を `is_main = false` と
/// して返す（旧 awk `__fzf_worktree_list` の `idx == 0` 判定と同じ）。
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

/// cdabbr の省略パスのベース種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbbrBase {
    /// `~` 始まり → `$HOME` から展開。
    Home,
    /// `/` 始まり → ルートから展開。
    Root,
}

/// prompt_pwd 風の省略パスを `(ベース種別, 空でないセグメント列)` に分解する純関数。
/// `~` / `/` 始まりでなければ `None`（呼び出し側がエラーにする）。旧 `cdabbr` の
/// 先頭判定 + `string split '/'` + 先頭要素除去 + 空要素除去に対応する。
pub fn parse_abbr_path(abbr: &str) -> Option<(AbbrBase, Vec<String>)> {
    let mut parts = abbr.split('/');
    let first = parts.next()?;
    let base = if first.starts_with('~') {
        AbbrBase::Home
    } else if first.is_empty() {
        // "/foo" は ["", "foo"] に割れるので先頭が空ならルート。
        AbbrBase::Root
    } else {
        return None;
    };
    let segments = parts
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    Some((base, segments))
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
        assert_eq!(
            parse_worktrees(input),
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
        assert_eq!(parse_worktrees(input)[0].label, "feature/deep/name");
    }

    #[test]
    fn missing_branch_and_not_detached_yields_empty_label() {
        let input = "worktree /w\nHEAD abc\n";
        assert_eq!(parse_worktrees(input)[0].label, "");
    }

    #[test]
    fn parse_abbr_path_home_and_root() {
        assert_eq!(
            parse_abbr_path("~/dev/foo"),
            Some((AbbrBase::Home, vec!["dev".to_string(), "foo".to_string()]))
        );
        assert_eq!(
            parse_abbr_path("/etc/nginx"),
            Some((AbbrBase::Root, vec!["etc".to_string(), "nginx".to_string()]))
        );
    }

    #[test]
    fn parse_abbr_path_bare_base_has_no_segments() {
        assert_eq!(parse_abbr_path("~"), Some((AbbrBase::Home, vec![])));
        assert_eq!(parse_abbr_path("/"), Some((AbbrBase::Root, vec![])));
    }

    #[test]
    fn parse_abbr_path_rejects_relative() {
        assert_eq!(parse_abbr_path("foo/bar"), None);
        assert_eq!(parse_abbr_path("./x"), None);
    }
}
