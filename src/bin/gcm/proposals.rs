//! `gcm`（AI コミット）の純粋ロジック。
//!
//! `claude --output-format json` が返す envelope から commit 提案を取り出す
//! 部分を切り出してユニットテスト対象にする。

use serde::{Deserialize, Serialize};

/// 1 コミットの提案。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub message: String,
    pub files: Vec<String>,
}

/// `--output-format json` の結果 envelope。gcm が参照するフィールドだけ拾う。
#[derive(Deserialize)]
struct Envelope {
    is_error: bool,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    structured_output: Option<StructuredOutput>,
}

/// スキーマで強制した構造化出力。
#[derive(Deserialize)]
struct StructuredOutput {
    commits: Vec<Commit>,
}

/// claude の envelope から commit 提案の配列を取り出す。
///
/// `is_error` が立っていれば `errors` を連結して失敗を返す。成功時は
/// `structured_output.commits` を返す（欠けていれば失敗）。
pub fn parse_proposals(raw: &str) -> Result<Vec<Commit>, String> {
    let envelope: Envelope =
        serde_json::from_str(raw.trim()).map_err(|e| format!("invalid JSON: {e}"))?;

    if envelope.is_error {
        return Err(if envelope.errors.is_empty() {
            "claude reported an error".to_string()
        } else {
            envelope.errors.join("; ")
        });
    }

    envelope
        .structured_output
        .map(|output| output.commits)
        .ok_or_else(|| "missing structured_output".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_envelope() {
        let raw = r#"{"type":"result","is_error":false,"structured_output":{"commits":[{"message":"feat: x","files":["a"]}]}}"#;
        let got = parse_proposals(raw).unwrap();
        assert_eq!(
            got,
            vec![Commit {
                message: "feat: x".into(),
                files: vec!["a".into()],
            }]
        );
    }

    #[test]
    fn parses_multiple_commits() {
        let raw = r#"{"is_error":false,"structured_output":{"commits":[{"message":"feat: a","files":["a"]},{"message":"fix: b","files":["b","c"]}]}}"#;
        let got = parse_proposals(raw).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[1].files, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn error_envelope_returns_error_message() {
        let raw = r#"{"is_error":true,"errors":["Reached maximum budget ($0.0001)"]}"#;
        let err = parse_proposals(raw).unwrap_err();
        assert!(err.contains("budget"));
    }

    #[test]
    fn missing_structured_output_is_error() {
        assert!(parse_proposals(r#"{"is_error":false}"#).is_err());
    }

    #[test]
    fn malformed_structured_output_is_error() {
        assert!(parse_proposals(r#"{"is_error":false,"structured_output":42}"#).is_err());
    }

    #[test]
    fn invalid_json_is_error() {
        assert!(parse_proposals("not json at all").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_proposals("").is_err());
    }
}
