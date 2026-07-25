//! GitHub リリースノートの取得。

use std::process::Command;

use serde::Deserialize;

use super::upstream::Repo;

/// GitHub の1リリース（本文とページ URL）。
pub struct ReleaseNotes {
    pub body: String,
    pub url: String,
}

/// 最新リリース（タグ省略）の本文と URL を取得する。
///
/// `current`→`latest` 間に複数リリースがあっても集約はしない
/// （パッケージマネージャのバージョン文字列と GitHub タグの命名規則を機械的に突き合わせる
/// 処理は信頼できないため。直近の変更として最新リリース1件を要約すれば実用上十分）。
pub fn fetch(repo: &Repo) -> Option<ReleaseNotes> {
    let output = Command::new("gh")
        .args([
            "release",
            "view",
            "--repo",
            &repo.slug(),
            "--json",
            "body,url",
        ])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| parse(&String::from_utf8_lossy(&output.stdout)))
        .flatten()
}

#[derive(Deserialize)]
struct GhReleaseView {
    body: String,
    url: String,
}

/// `gh release view --json body,url` の出力から [`ReleaseNotes`] を作る。本文が空なら
/// 要約する材料が無いので `None` にする。
fn parse(raw: &str) -> Option<ReleaseNotes> {
    let view: GhReleaseView = serde_json::from_str(raw.trim()).ok()?;
    let body = view.body.trim().to_string();
    (!body.is_empty()).then_some(ReleaseNotes {
        body,
        url: view.url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_notes() {
        let raw = r#"{"body":"What's Changed","url":"https://github.com/rustsec/rustsec/releases/tag/v1.0.0"}"#;
        let got = parse(raw).unwrap();
        assert_eq!(got.body, "What's Changed");
        assert_eq!(
            got.url,
            "https://github.com/rustsec/rustsec/releases/tag/v1.0.0"
        );
    }

    #[test]
    fn empty_body_is_none() {
        let raw = r#"{"body":"","url":"https://github.com/rustsec/rustsec/releases/tag/v1.0.0"}"#;
        assert!(parse(raw).is_none());
    }

    #[test]
    fn whitespace_only_body_is_none() {
        let raw = r#"{"body":"  \n ","url":"https://github.com/x/y/releases/tag/v1"}"#;
        assert!(parse(raw).is_none());
    }

    #[test]
    fn invalid_release_json_is_none() {
        assert!(parse("not json").is_none());
    }
}
