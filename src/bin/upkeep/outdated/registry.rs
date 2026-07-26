//! パッケージレジストリへの `repository` 問い合わせ。

use std::process::Command;

use serde::Deserialize;

/// crates.io がパッケージの `repository` として名指しする URL。
pub fn crates_io(name: &str) -> Option<String> {
    let raw = fetch(&format!("https://crates.io/api/v1/crates/{name}"))?;
    parse_crates_io(&raw)
}

/// npm registry がパッケージの `repository.url` として名指しする URL。
pub fn npm(name: &str) -> Option<String> {
    let raw = fetch(&format!("https://registry.npmjs.org/{name}"))?;
    parse_npm(&raw)
}

/// `User-Agent` が無いと crates.io が 403 を返すので必ず付ける。
fn fetch(url: &str) -> Option<String> {
    let output = Command::new("curl")
        .args([
            "-sS",
            "-f",
            "--max-time",
            "10",
            "-H",
            "User-Agent: dotfiles-upkeep (https://github.com/toshiki670/dotfiles)",
            url,
        ])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
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

fn parse_crates_io(raw: &str) -> Option<String> {
    serde_json::from_str::<CrateResponse>(raw)
        .ok()?
        .krate
        .repository
}

#[derive(Deserialize)]
struct NpmResponse {
    repository: Option<NpmRepository>,
}

/// npm の `repository` はオブジェクト形と短縮文字列形の両方が使われる。
#[derive(Deserialize)]
#[serde(untagged)]
enum NpmRepository {
    Object { url: String },
    Shorthand(String),
}

fn parse_npm(raw: &str) -> Option<String> {
    match serde_json::from_str::<NpmResponse>(raw).ok()?.repository? {
        NpmRepository::Object { url } => Some(url),
        NpmRepository::Shorthand(url) => Some(url),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_crates_io_repository() {
        let raw = r#"{"crate":{"repository":"https://github.com/rustsec/rustsec"}}"#;
        assert_eq!(
            parse_crates_io(raw),
            Some("https://github.com/rustsec/rustsec".to_string())
        );
    }

    #[test]
    fn missing_crates_io_repository_is_none() {
        assert_eq!(parse_crates_io(r#"{"crate":{}}"#), None);
    }

    #[test]
    fn null_crates_io_repository_is_none() {
        assert_eq!(parse_crates_io(r#"{"crate":{"repository":null}}"#), None);
    }

    #[test]
    fn invalid_crates_io_json_is_none() {
        assert_eq!(parse_crates_io("not json"), None);
    }

    #[test]
    fn extracts_npm_repository_object() {
        let raw =
            r#"{"repository":{"type":"git","url":"git+https://github.com/textlint/textlint.git"}}"#;
        assert_eq!(
            parse_npm(raw),
            Some("git+https://github.com/textlint/textlint.git".to_string())
        );
    }

    #[test]
    fn extracts_npm_repository_shorthand() {
        let raw = r#"{"repository":"github:sindresorhus/got"}"#;
        assert_eq!(parse_npm(raw), Some("github:sindresorhus/got".to_string()));
    }

    #[test]
    fn missing_npm_repository_is_none() {
        assert_eq!(parse_npm(r#"{"name":"x"}"#), None);
    }

    #[test]
    fn invalid_npm_json_is_none() {
        assert_eq!(parse_npm("not json"), None);
    }
}
