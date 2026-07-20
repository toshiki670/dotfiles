//! `mise outdated --json` の検出。
//!
//! `--bump` は付けない: `upkeep upgrade` は `mise upgrade` しか呼ばず、`mise upgrade`
//! は設定の制約内でしか上げない。`--bump` を付けると「upgrade しても変わらないもの」
//! まで拾ってしまい `upgrade.rs` の実際の挙動と矛盾する。

use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;

use super::package::{OutdatedPackage, Source};

#[derive(Deserialize)]
struct MiseEntry {
    current: String,
    latest: String,
}

/// `mise outdated --json` の stdout（JSON 文字列）から [`OutdatedPackage`] を作る。
///
/// トップレベルはオブジェクト（キー=ツール名）。該当なしは `{}`。
fn parse(raw: &str) -> Result<Vec<OutdatedPackage>, String> {
    let parsed: HashMap<String, MiseEntry> =
        serde_json::from_str(raw).map_err(|e| format!("invalid mise outdated JSON: {e}"))?;

    Ok(parsed
        .into_iter()
        .map(|(name, entry)| OutdatedPackage {
            source: Source::Mise,
            name,
            current: entry.current,
            latest: entry.latest,
        })
        .collect())
}

/// `mise outdated --json` を実行してアップデート可能なツールを集める。
///
/// 実行失敗・JSON パース失敗は警告して空の結果を返す（呼び出し元は続行する）。
pub fn detect() -> Vec<OutdatedPackage> {
    let output = match Command::new("mise").args(["outdated", "--json"]).output() {
        Ok(output) if output.status.success() => output,
        _ => {
            eprintln!("⚠️  mise outdated failed, skipping...");
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
    fn parses_single_entry() {
        let raw = r#"{"jq":{"name":"jq","requested":"1.6","current":"1.6","bump":"1.8","latest":"1.8.2","source":{"type":"mise.toml","path":"/tmp/mise.toml"}}}"#;
        let got = parse(raw).unwrap();
        assert_eq!(
            got,
            vec![OutdatedPackage {
                source: Source::Mise,
                name: "jq".into(),
                current: "1.6".into(),
                latest: "1.8.2".into(),
            }]
        );
    }

    #[test]
    fn parses_multiple_entries() {
        let raw = r#"{"jq":{"name":"jq","requested":"1.6","current":"1.6","bump":"1.8","latest":"1.8.2","source":{"type":"mise.toml","path":"/tmp/mise.toml"}},"node":{"name":"node","requested":"20","current":"20.1.0","bump":"20.2.0","latest":"20.2.0","source":{"type":"mise.toml","path":"/tmp/mise.toml"}}}"#;
        let got = parse(raw).unwrap();
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn empty_object_yields_empty_result() {
        assert_eq!(parse("{}").unwrap(), Vec::new());
    }

    #[test]
    fn invalid_json_is_error() {
        assert!(parse("not json at all").is_err());
    }
}
