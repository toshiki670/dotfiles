//! gh ピッカーのドメイン（Issue/PR を fzf で選び、種別に応じたアクションを組み立てる）。
//!
//! gh は worktree/ghq とは別ドメインなので、[`super::format`] / [`super::parse`]
//! （worktree 色のモジュール）には混ぜず、この 1 ファイルに凝集させる
//! （[`super::worktree`] がドメイン型 + 一覧取得 IO を同居させるのと同じ方針）。
//!
//! 純ロジック（選択行パース・アクション集合・コマンド組立）はユニットテストを同居させ、
//! `gh` を叩く IO（[`list_candidates`]）はバイナリの E2E（`tests/e2e/fzf_gh.rs`）で検証する。

use std::io;
use std::process::Command;

/// fzf 各行のフィールド区切り（表示=1列目、kind=2列目、番号=3列目）。
const TAB: char = '\t';

/// 選択対象の種別。`gh <group> <action> <number>` の group を決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Issue,
    Pr,
}

impl Kind {
    /// gh のサブコマンド group 名（`gh issue …` / `gh pr …`）。
    pub fn group(self) -> &'static str {
        match self {
            Kind::Issue => "issue",
            Kind::Pr => "pr",
        }
    }

    /// 種別ごとに選べるアクション。先頭を `view` にして「選ぶ→Enter」を最短にする。
    /// gh.fish.tmpl の Tab 補完と同じ集合に揃える。
    pub fn actions(self) -> &'static [&'static str] {
        match self {
            Kind::Issue => &["view", "edit", "close", "reopen", "comment", "pin"],
            Kind::Pr => &[
                "view",
                "checkout",
                "review",
                "diff",
                "merge",
                "checks",
                "update-branch",
            ],
        }
    }

    /// fzf 行の kind 列（`issue` / `pr`）を [`Kind`] に戻す。未知の種別は `None`。
    pub fn parse(s: &str) -> Option<Kind> {
        match s {
            "issue" => Some(Kind::Issue),
            "pr" => Some(Kind::Pr),
            _ => None,
        }
    }
}

/// fzf が返した選択行（`表示\tkind\t番号`）から `(Kind, 番号)` を取り出す。
///
/// 末尾 2 列の `kind \t 番号` だけに依存し、**末尾から**読む。表示列（先頭）の title に
/// 万一タブが紛れて列が増えても、kind/番号 を誤読しない（[`jq_for`] でも title を
/// サニタイズしているので二重の安全策）。表示列が無い（最低 3 列に満たない）・種別が
/// 未知・番号が空はいずれも `None`。
pub fn parse_selection(line: &str) -> Option<(Kind, String)> {
    let mut cols = line.rsplit(TAB);
    let number = cols.next()?.trim();
    let kind = Kind::parse(cols.next()?)?;
    cols.next()?; // 表示列が存在すること（最低 3 列）。
    (!number.is_empty()).then(|| (kind, number.to_string()))
}

/// `gh <group> <action> <number>` を組み立てる（fish へ挿入する 1 行）。
pub fn build_command(kind: Kind, action: &str, number: &str) -> String {
    format!("gh {} {} {}", kind.group(), action, number)
}

/// Issue と PR を `表示\tkind\t番号` の fzf 候補行にして連結して返す。
///
/// 表示列は内部タブ無しの 1 列（番号・title・author・assignee を含む）なので、`run_fzf`
/// の `--with-nth 1` でそれら全てが検索対象になる。表示成形は `gh` の `--jq` に委譲する。
pub fn list_candidates() -> io::Result<Vec<String>> {
    let mut lines = list_one(Kind::Issue)?;
    lines.extend(list_one(Kind::Pr)?);
    Ok(lines)
}

/// 片側（Issue または PR）の候補行を取得する。`gh` が repo 外・未認証などで失敗しても、
/// もう片方は出したいので空 Vec を返す（呼び出し側でまとめてキャンセル扱いにできる）。
fn list_one(kind: Kind) -> io::Result<Vec<String>> {
    let jq = jq_for(kind);
    let output = Command::new("gh")
        .args([
            kind.group(),
            "list",
            "--limit",
            "100",
            "--json",
            "number,title,author,assignees",
            "--jq",
            &jq,
        ])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// `gh … --jq` に渡す式。1 行を `[group] #番号 title @author →assignees\tgroup\t番号` に
/// 成形する（assignee が無ければ ` →…` 部分を省く）。
///
/// `title` はタブ・改行を含み得るので空白へ置換しておく。これを怠ると、表示列の中に
/// タブが入って列がずれたり、改行で 1 候補が 2 行に割れたりする（番号/種別は GitHub の
/// 性質上タブ・改行を含まないので対象外）。
fn jq_for(kind: Kind) -> String {
    let group = kind.group();
    format!(
        r#".[] | "[{group}] #\(.number) \(.title | gsub("[\t\n\r]"; " ")) @\(.author.login)\(if (.assignees | length) > 0 then " →" + ([.assignees[].login] | join(",")) else "" end)\t{group}\t\(.number)""#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_names() {
        assert_eq!(Kind::Issue.group(), "issue");
        assert_eq!(Kind::Pr.group(), "pr");
    }

    #[test]
    fn parse_kind_roundtrip_and_unknown() {
        assert_eq!(Kind::parse("issue"), Some(Kind::Issue));
        assert_eq!(Kind::parse("pr"), Some(Kind::Pr));
        assert_eq!(Kind::parse("gist"), None);
    }

    #[test]
    fn actions_lead_with_view() {
        assert_eq!(Kind::Issue.actions().first(), Some(&"view"));
        assert!(Kind::Issue.actions().contains(&"edit"));
        assert_eq!(Kind::Pr.actions().first(), Some(&"view"));
        assert!(Kind::Pr.actions().contains(&"checkout"));
    }

    #[test]
    fn parse_selection_extracts_issue() {
        let line = "[issue] #341 demo @me\tissue\t341";
        assert_eq!(
            parse_selection(line),
            Some((Kind::Issue, "341".to_string()))
        );
    }

    #[test]
    fn parse_selection_extracts_pr() {
        let line = "[pr] #7 fix @you\tpr\t7";
        assert_eq!(parse_selection(line), Some((Kind::Pr, "7".to_string())));
    }

    #[test]
    fn parse_selection_rejects_missing_columns_or_unknown_kind() {
        assert_eq!(parse_selection("only display"), None);
        assert_eq!(parse_selection("display\tissue"), None);
        assert_eq!(parse_selection("display\tunknown\t9"), None);
    }

    #[test]
    fn parse_selection_rejects_empty_number() {
        assert_eq!(parse_selection("display\tissue\t"), None);
    }

    #[test]
    fn parse_selection_survives_tab_in_display() {
        // title 由来のタブで表示列が割れても、末尾から読むので kind/番号 は正しく取れる。
        let line = "[issue] #341 weird\ttitle\tissue\t341";
        assert_eq!(
            parse_selection(line),
            Some((Kind::Issue, "341".to_string()))
        );
    }

    #[test]
    fn build_command_formats_invocation() {
        assert_eq!(
            build_command(Kind::Issue, "edit", "341"),
            "gh issue edit 341"
        );
        assert_eq!(build_command(Kind::Pr, "checkout", "7"), "gh pr checkout 7");
    }
}
