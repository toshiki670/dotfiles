//! `mise outdated --json` の検出と、backend からの上流解決。
//!
//! `--bump` は付けない: `upkeep upgrade` は `mise upgrade` しか呼ばず、`mise upgrade`
//! は設定の制約内でしか上げない。`--bump` を付けると「upgrade しても変わらないもの」
//! まで拾ってしまい `upgrade.rs` の実際の挙動と矛盾する。

use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;

use super::package::{OutdatedPackage, Source};
use super::registry;
use super::upstream::{Repo, Upstream};

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

/// `core:` backend が指すツールの正典リポジトリ。
///
/// `core:` は mise 組み込みの固定集合で、backend 文字列がリポジトリを名指ししないので
/// ここで対応を持つ。`java`（Adoptium / Zulu 等のビルドを入れる）と `dotnet`（SDK と
/// ランタイムで別）は指すべきリポジトリが一つに定まらないため持たない。
const CORE_REPOS: &[(&str, &str)] = &[
    ("bun", "oven-sh/bun"),
    ("deno", "denoland/deno"),
    ("elixir", "elixir-lang/elixir"),
    ("erlang", "erlang/otp"),
    ("go", "golang/go"),
    ("node", "nodejs/node"),
    ("python", "python/cpython"),
    ("ruby", "ruby/ruby"),
    ("rust", "rust-lang/rust"),
    ("swift", "swiftlang/swift"),
    ("zig", "ziglang/zig"),
];

#[derive(Deserialize)]
struct MiseTool {
    backend: String,
}

/// `mise tool <name> --json` の出力から backend 文字列を取り出す。
fn parse_backend(raw: &str) -> Option<String> {
    serde_json::from_str::<MiseTool>(raw.trim())
        .ok()
        .map(|tool| tool.backend)
}

/// backend 文字列から上流を引く。
///
/// `asdf:` / `vfox:` は対象外にする。これらが名指すのはツール本体ではなく**プラグインの
/// リポジトリ**（`asdf:luizm/asdf-shfmt`）で、辿ると無関係なリリースノートを要約してしまう。
/// aqua は第1セグメントにドットを含む形（`aqua:atlassian.com/acli`）が GitHub 以外の
/// 配布元を指すので除く。
fn upstream_for_backend(backend: &str) -> Upstream {
    let Some((kind, spec)) = backend.split_once(':') else {
        return Upstream::default();
    };

    let repo = match kind {
        "aqua" | "github" | "ubi" => Repo::from_slug(spec).filter(|r| !r.owner.contains('.')),
        "core" => CORE_REPOS
            .iter()
            .find(|(tool, _)| *tool == spec)
            .and_then(|(_, slug)| Repo::from_slug(slug)),
        "cargo" => registry::crates_io(spec)
            .as_deref()
            .and_then(Repo::from_url),
        "npm" => registry::npm(spec).as_deref().and_then(repo_from_npm),
        _ => None,
    };

    Upstream {
        repo,
        homepage: None,
    }
}

/// npm の `repository` は URL 形と `github:<owner>/<repo>` の短縮形の両方が使われる。
fn repo_from_npm(url: &str) -> Option<Repo> {
    Repo::from_url(url).or_else(|| Repo::from_slug(url.strip_prefix("github:")?))
}

/// インストール済みツールの backend から上流を引く。
///
/// 実行失敗（レジストリに無いツール等）は解決なしとして扱う。候補を複数返す
/// `mise registry` ではなく `mise tool` を使う: インストール済みの backend が一つに定まる。
pub fn upstream(name: &str) -> Upstream {
    let output = match Command::new("mise").args(["tool", name, "--json"]).output() {
        Ok(output) if output.status.success() => output,
        _ => return Upstream::default(),
    };

    parse_backend(&String::from_utf8_lossy(&output.stdout))
        .as_deref()
        .map(upstream_for_backend)
        .unwrap_or_default()
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

    #[test]
    fn extracts_backend() {
        let raw = r#"{"backend":"aqua:jqlang/jq","description":"Command-line JSON processor"}"#;
        assert_eq!(parse_backend(raw).as_deref(), Some("aqua:jqlang/jq"));
    }

    #[test]
    fn missing_backend_is_none() {
        assert_eq!(parse_backend(r#"{"description":"x"}"#), None);
    }

    #[test]
    fn invalid_tool_json_is_none() {
        assert_eq!(parse_backend("not json at all"), None);
    }

    #[test]
    fn aqua_backend_resolves_repo() {
        assert_eq!(
            upstream_for_backend("aqua:jqlang/jq").repo,
            repo("jqlang", "jq")
        );
    }

    #[test]
    fn github_backend_resolves_repo() {
        assert_eq!(
            upstream_for_backend("github:rvben/rumdl").repo,
            repo("rvben", "rumdl")
        );
    }

    #[test]
    fn aqua_domain_form_is_not_github() {
        assert_eq!(upstream_for_backend("aqua:atlassian.com/acli").repo, None);
    }

    #[test]
    fn core_backend_resolves_mapped_repo() {
        assert_eq!(
            upstream_for_backend("core:node").repo,
            repo("nodejs", "node")
        );
    }

    #[test]
    fn core_backend_without_canonical_repo_resolves_nothing() {
        assert_eq!(upstream_for_backend("core:java").repo, None);
        assert_eq!(upstream_for_backend("core:dotnet").repo, None);
    }

    #[test]
    fn plugin_backends_are_skipped() {
        // プラグインのリポジトリであってツール本体ではない。
        assert_eq!(upstream_for_backend("asdf:luizm/asdf-shfmt").repo, None);
        assert_eq!(
            upstream_for_backend("vfox:mise-plugins/vfox-1password").repo,
            None
        );
    }

    #[test]
    fn unknown_backend_resolves_nothing() {
        assert_eq!(upstream_for_backend("conda:numpy").repo, None);
    }

    #[test]
    fn backend_without_separator_resolves_nothing() {
        assert_eq!(upstream_for_backend("bogus").repo, None);
    }

    #[test]
    fn npm_repository_url_form() {
        assert_eq!(
            repo_from_npm("git+https://github.com/textlint/textlint.git"),
            repo("textlint", "textlint")
        );
    }

    #[test]
    fn npm_repository_shorthand_form() {
        assert_eq!(
            repo_from_npm("github:sindresorhus/got"),
            repo("sindresorhus", "got")
        );
    }

    #[test]
    fn npm_repository_on_other_host_is_none() {
        assert_eq!(repo_from_npm("https://gitlab.com/owner/repo"), None);
    }
}
