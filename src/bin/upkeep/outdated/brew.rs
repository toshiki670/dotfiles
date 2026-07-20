//! `brew outdated --json=v2` の検出。formulae/casks は同一シェイプなので共通デコードする。

use std::process::Command;

use serde::Deserialize;

use super::package::{OutdatedPackage, Source};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
