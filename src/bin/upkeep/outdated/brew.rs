//! `brew outdated --json=v2` の検出と、`brew info --json=v2` からの上流解決。

use std::process::Command;

use serde::Deserialize;

use super::package::{OutdatedPackage, Source};
use super::upstream::{Repo, Upstream};

#[derive(Deserialize)]
struct BrewOutdatedV2 {
    #[serde(default)]
    formulae: Vec<BrewItem>,
    #[serde(default)]
    casks: Vec<BrewItem>,
}

#[derive(Deserialize)]
struct BrewItem {
    name: String,
    installed_versions: Vec<String>,
    current_version: String,
}

/// `brew outdated --json=v2` の stdout（JSON 文字列）から [`OutdatedPackage`] を作る。
///
/// `installed_versions` は複数バージョン共存しうるが、先頭要素を「現在の version」とする。
fn parse(raw: &str) -> Result<Vec<OutdatedPackage>, String> {
    let parsed: BrewOutdatedV2 =
        serde_json::from_str(raw).map_err(|e| format!("invalid brew outdated JSON: {e}"))?;

    let to_pkg = |item: BrewItem| OutdatedPackage {
        source: Source::Brew,
        name: item.name,
        current: item.installed_versions.first().cloned().unwrap_or_default(),
        latest: item.current_version,
    };

    Ok(parsed
        .formulae
        .into_iter()
        .chain(parsed.casks)
        .map(to_pkg)
        .collect())
}

/// `brew outdated --json=v2` を実行してアップデート可能な formula/cask を集める。
///
/// 実行失敗・JSON パース失敗は警告して空の結果を返す（呼び出し元は続行する）。
pub fn detect() -> Vec<OutdatedPackage> {
    let output = match Command::new("brew")
        .args(["outdated", "--json=v2"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => {
            eprintln!("⚠️  brew outdated failed, skipping...");
            return Vec::new();
        }
    };

    match parse(&String::from_utf8_lossy(&output.stdout)) {
        Ok(packages) => packages,
        Err(msg) => {
            eprintln!("⚠️  {msg}");
            Vec::new()
        }
    }
}

#[derive(Deserialize)]
struct BrewInfoV2 {
    #[serde(default)]
    formulae: Vec<FormulaInfo>,
    #[serde(default)]
    casks: Vec<CaskInfo>,
}

#[derive(Deserialize)]
struct FormulaInfo {
    homepage: Option<String>,
    urls: Option<FormulaUrls>,
}

#[derive(Deserialize)]
struct FormulaUrls {
    stable: Option<FormulaUrl>,
}

#[derive(Deserialize)]
struct FormulaUrl {
    url: Option<String>,
}

#[derive(Deserialize)]
struct CaskInfo {
    homepage: Option<String>,
    url: Option<String>,
}

/// `brew info --json=v2` の stdout から上流を組み立てる。
///
/// リポジトリは配布物 URL → homepage の順に GitHub を探す。cask の homepage は製品サイト
/// （`https://codexbar.app/`）でリポジトリを指さないことが多い一方、配布物 URL は
/// GitHub リリース資産を直に指すため、配布物 URL を先に見る。
fn parse_info(raw: &str) -> Upstream {
    let Ok(info) = serde_json::from_str::<BrewInfoV2>(raw) else {
        return Upstream::default();
    };

    let (homepage, download_url) = match (
        info.formulae.into_iter().next(),
        info.casks.into_iter().next(),
    ) {
        (Some(formula), _) => (
            formula.homepage,
            formula
                .urls
                .and_then(|urls| urls.stable)
                .and_then(|stable| stable.url),
        ),
        (None, Some(cask)) => (cask.homepage, cask.url),
        (None, None) => return Upstream::default(),
    };

    Upstream {
        repo: download_url
            .as_deref()
            .and_then(Repo::from_url)
            .or_else(|| homepage.as_deref().and_then(Repo::from_url)),
        homepage,
    }
}

/// formula/cask のメタデータから上流を引く。
///
/// 実行失敗（未知のパッケージ等）は解決なしとして扱う（呼び出し元は要約なしで続行する）。
pub fn upstream(name: &str) -> Upstream {
    let output = match Command::new("brew")
        .args(["info", "--json=v2", name])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return Upstream::default(),
    };

    parse_info(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(owner: &str, name: &str) -> Option<Repo> {
        Some(Repo {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }

    #[test]
    fn parses_formulae_and_casks() {
        let raw = r#"{"formulae":[{"name":"bat","installed_versions":["0.24.0"],"current_version":"0.25.0","pinned":false,"pinned_version":null}],"casks":[{"name":"codexbar","installed_versions":["0.45.0"],"current_version":"0.45.2","pinned":false,"pinned_version":null}]}"#;
        let got = parse(raw).unwrap();
        assert_eq!(
            got,
            vec![
                OutdatedPackage {
                    source: Source::Brew,
                    name: "bat".into(),
                    current: "0.24.0".into(),
                    latest: "0.25.0".into(),
                },
                OutdatedPackage {
                    source: Source::Brew,
                    name: "codexbar".into(),
                    current: "0.45.0".into(),
                    latest: "0.45.2".into(),
                },
            ]
        );
    }

    #[test]
    fn parses_formulae_only() {
        let raw = r#"{"formulae":[{"name":"bat","installed_versions":["0.24.0"],"current_version":"0.25.0","pinned":false,"pinned_version":null}],"casks":[]}"#;
        let got = parse(raw).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "bat");
    }

    #[test]
    fn parses_casks_only() {
        let raw = r#"{"formulae":[],"casks":[{"name":"codexbar","installed_versions":["0.45.0"],"current_version":"0.45.2","pinned":false,"pinned_version":null}]}"#;
        let got = parse(raw).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "codexbar");
    }

    #[test]
    fn empty_arrays_yield_empty_result() {
        let raw = r#"{"formulae":[],"casks":[]}"#;
        assert_eq!(parse(raw).unwrap(), Vec::new());
    }

    #[test]
    fn invalid_json_is_error() {
        assert!(parse("not json at all").is_err());
    }

    #[test]
    fn formula_resolves_repo_from_stable_url() {
        let raw = r#"{"formulae":[{"homepage":"https://github.com/sharkdp/bat","urls":{"stable":{"url":"https://github.com/sharkdp/bat/archive/refs/tags/v0.26.1.tar.gz"}}}],"casks":[]}"#;
        let got = parse_info(raw);
        assert_eq!(got.repo, repo("sharkdp", "bat"));
        assert_eq!(
            got.homepage.as_deref(),
            Some("https://github.com/sharkdp/bat")
        );
    }

    #[test]
    fn cask_resolves_repo_from_download_url_not_product_site() {
        let raw = r#"{"formulae":[],"casks":[{"homepage":"https://codexbar.app/","url":"https://github.com/steipete/CodexBar/releases/download/v0.45.2/CodexBar-macos-universal-0.45.2.zip"}]}"#;
        let got = parse_info(raw);
        assert_eq!(got.repo, repo("steipete", "CodexBar"));
        assert_eq!(got.homepage.as_deref(), Some("https://codexbar.app/"));
    }

    #[test]
    fn falls_back_to_homepage_when_download_url_is_not_github() {
        let raw = r#"{"formulae":[{"homepage":"https://github.com/owner/repo","urls":{"stable":{"url":"https://mirror.example.com/pkg-1.0.tar.gz"}}}],"casks":[]}"#;
        assert_eq!(parse_info(raw).repo, repo("owner", "repo"));
    }

    #[test]
    fn non_github_upstream_keeps_homepage_without_repo() {
        let raw = r#"{"formulae":[{"homepage":"https://ffmpeg.org/","urls":{"stable":{"url":"https://ffmpeg.org/releases/ffmpeg-8.0.tar.xz"}}}],"casks":[]}"#;
        let got = parse_info(raw);
        assert_eq!(got.repo, None);
        assert_eq!(got.homepage.as_deref(), Some("https://ffmpeg.org/"));
    }

    #[test]
    fn formula_wins_when_a_name_matches_both_kinds() {
        let raw = r#"{"formulae":[{"homepage":"https://github.com/owner/formula","urls":null}],"casks":[{"homepage":"https://github.com/owner/cask","url":null}]}"#;
        assert_eq!(parse_info(raw).repo, repo("owner", "formula"));
    }

    #[test]
    fn missing_urls_falls_back_to_homepage() {
        let raw = r#"{"formulae":[{"homepage":"https://github.com/owner/repo"}],"casks":[]}"#;
        assert_eq!(parse_info(raw).repo, repo("owner", "repo"));
    }

    #[test]
    fn empty_info_resolves_nothing() {
        assert_eq!(
            parse_info(r#"{"formulae":[],"casks":[]}"#),
            Upstream::default()
        );
    }

    #[test]
    fn invalid_info_json_resolves_nothing() {
        assert_eq!(parse_info("not json at all"), Upstream::default());
    }
}
