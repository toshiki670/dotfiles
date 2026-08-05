//! GitHub リリースノートの取得。
//!
//! `current` の次から `latest` までを1回のリクエストでまとめて取る。どのタグがどの版かは
//! [`super::tag`] が突き合わせる。

use std::process::Command;

use serde::Deserialize;

use super::tag;
use super::upstream::Repo;

/// 1リクエストで取るリリース数。GitHub API の上限値で、ページを繰らない以上これが範囲の
/// 上限にもなる。これより古い版から更新すると [`Coverage::LatestOnly`] へ落ちる。
const PAGE_SIZE: usize = 100;

/// GitHub の1リリース（本文とページ URL）。
pub struct ReleaseNotes {
    pub tag: String,
    pub body: String,
    pub url: String,
}

/// 要約対象として切り出したリリース群。新しい順。
pub struct Span {
    pub notes: Vec<ReleaseNotes>,
    pub coverage: Coverage,
}

/// 切り出しが `current` を起点に確定できたか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// `current` の次から `latest` までを漏れなく拾えた。
    Complete,
    /// `current` を起点に据えられず、最新1件だけに絞った。間に何があったかは不明。
    ///
    /// タグの綴りが [`super::tag`] の想定から外れている場合と、`current` が
    /// [`PAGE_SIZE`] 件より古くて取得範囲に入らなかった場合がある。どちらかは区別しない
    /// ので、利用者へ原因は名乗らない。
    LatestOnly,
}

/// `current` の次から `latest` までのリリースノートを取る。
pub fn fetch(repo: &Repo, name: &str, current: &str, latest: &str) -> Option<Span> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/releases?per_page={PAGE_SIZE}", repo.slug()),
        ])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| {
            select(
                &String::from_utf8_lossy(&output.stdout),
                name,
                current,
                latest,
            )
        })
        .flatten()
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    /// 本文は未記入だと `null` で返る。
    #[serde(default)]
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// `gh api .../releases` の出力から要約対象を切り出す。
///
/// 正式リリースだけを並べてから範囲を決める。本文の空判定を先に持ってくると、`current`
/// 自身の本文が空だったときに起点を見失って [`Coverage::LatestOnly`] へ落ちてしまう。
fn select(raw: &str, name: &str, current: &str, latest: &str) -> Option<Span> {
    let releases: Vec<GhRelease> = serde_json::from_str(raw.trim()).ok()?;
    let published: Vec<GhRelease> = releases
        .into_iter()
        .filter(|release| !release.draft && !release.prerelease)
        .collect();
    if published.is_empty() {
        return None;
    }

    let find = |version: &str| {
        published
            .iter()
            .position(|release| tag::matches(&release.tag_name, version, name))
    };

    // `latest` が見つからなければ先頭（＝最新）から。配布元の版がまだリリース化されていない
    // ことがあるが、その場合も「今より新しいもの」を拾う起点としては先頭でよい。
    let start = find(latest).unwrap_or(0);
    // `current` 自身は既に入っているので、その手前で止める。
    let anchor = find(current).filter(|&index| index > start);
    let coverage = match anchor {
        Some(_) => Coverage::Complete,
        None => Coverage::LatestOnly,
    };
    let end = anchor.unwrap_or(start + 1).min(published.len());

    let notes: Vec<ReleaseNotes> = published[start..end]
        .iter()
        .filter_map(|release| {
            let body = release.body.as_deref().unwrap_or_default().trim();
            (!body.is_empty()).then(|| ReleaseNotes {
                tag: release.tag_name.clone(),
                body: body.to_string(),
                url: release.html_url.clone(),
            })
        })
        .collect();

    (!notes.is_empty()).then_some(Span { notes, coverage })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tag` と本文だけを差し替えた1リリース分の JSON。
    fn release(tag: &str, body: &str) -> String {
        entry(tag, body, false)
    }

    fn entry(tag: &str, body: &str, prerelease: bool) -> String {
        format!(
            r#"{{"tag_name":"{tag}","body":"{body}","html_url":"https://github.com/o/r/releases/tag/{tag}","draft":false,"prerelease":{prerelease}}}"#
        )
    }

    fn array(entries: &[String]) -> String {
        format!("[{}]", entries.join(","))
    }

    fn tags(span: &Span) -> Vec<&str> {
        span.notes.iter().map(|note| note.tag.as_str()).collect()
    }

    #[test]
    fn takes_every_release_after_current() {
        let raw = array(&[
            release("v1.3.0", "c"),
            release("v1.2.0", "b"),
            release("v1.1.0", "a"),
            release("v1.0.0", "old"),
        ]);
        let span = select(&raw, "pkg", "1.0.0", "1.3.0").unwrap();
        assert_eq!(tags(&span), ["v1.3.0", "v1.2.0", "v1.1.0"]);
        assert_eq!(span.coverage, Coverage::Complete);
    }

    #[test]
    fn adjacent_releases_yield_a_single_note() {
        let raw = array(&[release("v1.1.0", "new"), release("v1.0.0", "old")]);
        let span = select(&raw, "pkg", "1.0.0", "1.1.0").unwrap();
        assert_eq!(tags(&span), ["v1.1.0"]);
        assert_eq!(span.coverage, Coverage::Complete);
    }

    /// 範囲に挟まる prerelease は要約対象にしない（herdr の `preview-*` の形）。
    #[test]
    fn prereleases_inside_the_span_are_excluded() {
        let raw = array(&[
            entry("v0.8.0", "release", false),
            entry("preview-2026-07-29-44b3ad", "preview", true),
            entry("preview-2026-07-21-0f10e1", "preview", true),
            entry("v0.7.5", "old", false),
        ]);
        let span = select(&raw, "herdr", "0.7.5", "0.8.0").unwrap();
        assert_eq!(tags(&span), ["v0.8.0"]);
        assert_eq!(span.coverage, Coverage::Complete);
    }

    #[test]
    fn drafts_are_excluded() {
        let raw = format!(
            "[{},{}]",
            r#"{"tag_name":"v2.0.0","body":"wip","html_url":"https://github.com/o/r/releases/tag/v2.0.0","draft":true,"prerelease":false}"#,
            release("v1.0.0", "shipped")
        );
        let span = select(&raw, "pkg", "0.9.0", "1.0.0").unwrap();
        assert_eq!(tags(&span), ["v1.0.0"]);
    }

    /// `current` のタグを引けないときは最新1件へ落とし、範囲未確定であることを残す。
    #[test]
    fn falls_back_to_latest_when_current_tag_is_unknown() {
        let raw = array(&[release("v1.2.0", "b"), release("v1.1.0", "a")]);
        let span = select(&raw, "luajit", "2.1.1785606157", "1.2.0").unwrap();
        assert_eq!(tags(&span), ["v1.2.0"]);
        assert_eq!(span.coverage, Coverage::LatestOnly);
    }

    /// 配布元の版がまだリリース化されていなくても、`current` より新しい分は拾える。
    #[test]
    fn unknown_latest_tag_still_anchors_on_current() {
        let raw = array(&[release("v1.2.0", "b"), release("v1.1.0", "a")]);
        let span = select(&raw, "pkg", "1.1.0", "1.3.0").unwrap();
        assert_eq!(tags(&span), ["v1.2.0"]);
        assert_eq!(span.coverage, Coverage::Complete);
    }

    /// `current` 自身の本文が空でも、起点としては使える。
    #[test]
    fn empty_body_on_current_still_anchors_the_span() {
        let raw = array(&[release("v1.2.0", "b"), release("v1.1.0", "")]);
        let span = select(&raw, "pkg", "1.1.0", "1.2.0").unwrap();
        assert_eq!(tags(&span), ["v1.2.0"]);
        assert_eq!(span.coverage, Coverage::Complete);
    }

    /// 本文の無いリリースは要約する材料が無いので落とす。
    #[test]
    fn releases_without_a_body_are_dropped_from_the_span() {
        let raw = array(&[
            release("v1.3.0", "c"),
            release("v1.2.0", "   "),
            release("v1.1.0", "a"),
            release("v1.0.0", "old"),
        ]);
        let span = select(&raw, "pkg", "1.0.0", "1.3.0").unwrap();
        assert_eq!(tags(&span), ["v1.3.0", "v1.1.0"]);
    }

    #[test]
    fn all_bodies_empty_is_none() {
        let raw = array(&[release("v1.1.0", ""), release("v1.0.0", "old")]);
        assert!(select(&raw, "pkg", "1.0.0", "1.1.0").is_none());
    }

    #[test]
    fn no_releases_is_none() {
        assert!(select("[]", "cargo-modules", "0.26.0", "0.27.0").is_none());
    }

    #[test]
    fn only_prereleases_is_none() {
        let raw = array(&[entry("v1.0.0-rc1", "rc", true)]);
        assert!(select(&raw, "pkg", "0.9.0", "1.0.0").is_none());
    }

    #[test]
    fn invalid_json_is_none() {
        assert!(select("not json", "pkg", "1.0.0", "1.1.0").is_none());
    }

    #[test]
    fn null_body_is_treated_as_empty() {
        let raw = r#"[{"tag_name":"v1.0.0","body":null,"html_url":"https://github.com/o/r/releases/tag/v1.0.0","draft":false,"prerelease":false}]"#;
        assert!(select(raw, "pkg", "0.9.0", "1.0.0").is_none());
    }

    #[test]
    fn keeps_the_release_page_url() {
        let raw = array(&[release("v1.0.0", "note")]);
        let span = select(&raw, "pkg", "0.9.0", "1.0.0").unwrap();
        assert_eq!(
            span.notes[0].url,
            "https://github.com/o/r/releases/tag/v1.0.0"
        );
    }
}
