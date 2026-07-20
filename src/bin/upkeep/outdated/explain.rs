//! `--explain` のリポジトリ解決とリリースノート要約。
//!
//! 機械的にリポジトリを特定できるのは cargo バイナリ（crates.io の `repository` から
//! GitHub リポジトリを特定できる）だけ。brew（formula/cask）・mise backend は解決を
//! 試みず、常に [`Explanation::Unavailable`] を返す（過剰な推測はしない）。

use std::process::Command;

use serde::Deserialize;

use super::package::{OutdatedPackage, Source};

/// 1 パッケージについて `--explain` を試みた結果。
pub enum Explanation {
    /// リリースノート本文を機械的に解決できなかった
    /// （brew/mise は無条件、cargo でも repository 不明・GitHub 以外・本文取得失敗）。
    Unavailable,
    /// リリースノートは取得できたが claude による要約に失敗した。
    GenerationFailed,
    /// 要約成功。
    Summary(String),
}

/// パッケージのリリースノートを解決し、取得できれば claude で要約する。
///
/// cargo 以外は解決を試みず即 [`Explanation::Unavailable`] を返す。
pub fn resolve(pkg: &OutdatedPackage) -> Explanation {
    if pkg.source != Source::Cargo {
        return Explanation::Unavailable;
    }

    let Some(body) = release_notes_for_cargo_package(&pkg.name) else {
        return Explanation::Unavailable;
    };

    match super::claude::summarize(&body) {
        Some(summary) => Explanation::Summary(summary),
        None => Explanation::GenerationFailed,
    }
}

fn release_notes_for_cargo_package(name: &str) -> Option<String> {
    let raw = fetch_crate_metadata(name)?;
    let repo_url = extract_repository_url(&raw)?;
    let (owner, repo) = parse_owner_repo(&repo_url)?;
    fetch_release_body(&owner, &repo)
}

/// crates.io にパッケージのメタデータを問い合わせる。`User-Agent` が無いと 403 になる。
fn fetch_crate_metadata(name: &str) -> Option<String> {
    let output = Command::new("curl")
        .args([
            "-sS",
            "-f",
            "--max-time",
            "10",
            "-H",
            "User-Agent: dotfiles-upkeep (https://github.com/toshiki670/dotfiles)",
            &format!("https://crates.io/api/v1/crates/{name}"),
        ])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// GitHub の最新リリース（タグ省略）の本文を取得する。
///
/// `current`→`latest` 間に複数リリースがあっても集約はしない
/// （crates.io のバージョン文字列と GitHub タグの命名規則を機械的に突き合わせる処理は
/// 信頼できないため。直近の変更として最新リリース1件を要約すれば実用上十分）。
fn fetch_release_body(owner: &str, repo: &str) -> Option<String> {
    let output = Command::new("gh")
        .args([
            "release",
            "view",
            "--repo",
            &format!("{owner}/{repo}"),
            "--json",
            "body",
            "--jq",
            ".body",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let body = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!body.is_empty()).then_some(body)
}

#[derive(Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
}

#[derive(Deserialize)]
struct CrateInfo {
    repository: Option<String>,
}

/// crates.io レスポンスから `repository` を取り出す。
///
/// `crate` は Rust の予約語なので `#[serde(rename = "crate")]` でフィールド名をずらす。
fn extract_repository_url(raw: &str) -> Option<String> {
    serde_json::from_str::<CrateResponse>(raw)
        .ok()?
        .krate
        .repository
}

/// `https://github.com/<owner>/<repo>(.git)?(/...)?` から owner/repo を取り出す。
///
/// GitHub 以外のホスト（GitLab 等）は `None`（機械的に特定できない扱い）。
fn parse_owner_repo(url: &str) -> Option<(String, String)> {
    let rest = url.split("github.com/").nth(1)?;
    let mut parts = rest.trim_end_matches('/').splitn(3, '/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.trim_end_matches(".git").to_string();
    (!owner.is_empty() && !repo.is_empty()).then_some((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_repository_url() {
        let raw = r#"{"crate":{"repository":"https://github.com/rustsec/rustsec"}}"#;
        assert_eq!(
            extract_repository_url(raw),
            Some("https://github.com/rustsec/rustsec".to_string())
        );
    }

    #[test]
    fn missing_repository_is_none() {
        let raw = r#"{"crate":{}}"#;
        assert_eq!(extract_repository_url(raw), None);
    }

    #[test]
    fn null_repository_is_none() {
        let raw = r#"{"crate":{"repository":null}}"#;
        assert_eq!(extract_repository_url(raw), None);
    }

    #[test]
    fn invalid_json_is_none() {
        assert_eq!(extract_repository_url("not json"), None);
    }

    #[test]
    fn parses_plain_github_url() {
        assert_eq!(
            parse_owner_repo("https://github.com/rustsec/rustsec"),
            Some(("rustsec".to_string(), "rustsec".to_string()))
        );
    }

    #[test]
    fn parses_git_suffixed_url() {
        assert_eq!(
            parse_owner_repo("https://github.com/kbknapp/cargo-outdated.git"),
            Some(("kbknapp".to_string(), "cargo-outdated".to_string()))
        );
    }

    #[test]
    fn parses_monorepo_subpath_url() {
        assert_eq!(
            parse_owner_repo("https://github.com/owner/monorepo/tree/main/crates/pkg"),
            Some(("owner".to_string(), "monorepo".to_string()))
        );
    }

    #[test]
    fn non_github_host_is_none() {
        assert_eq!(parse_owner_repo("https://gitlab.com/owner/repo"), None);
    }

    #[test]
    fn malformed_url_is_none() {
        assert_eq!(parse_owner_repo("https://github.com/owner-only"), None);
    }
}
