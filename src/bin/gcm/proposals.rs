//! `gcm`（AI コミット）の純粋ロジック。
//!
//! claude の出力（markdown フェンスで包まれることがある）から commit 提案を
//! 取り出す部分を切り出してユニットテスト対象にする。単一オブジェクトで返ってきた
//! 場合も配列へ正規化する。

use serde::{Deserialize, Serialize};

/// 1 コミットの提案。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub message: String,
    pub files: Vec<String>,
}

/// markdown コードフェンス行（```... で始まる行）を除去する。
pub fn strip_fences(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// claude の生出力から commit 提案の配列を取り出す。
///
/// 単一オブジェクトは配列に正規化する。オブジェクト/配列以外、または
/// `message` / `files` を持たない要素は失敗。
pub fn parse_proposals(raw: &str) -> Result<Vec<Commit>, String> {
    let cleaned = strip_fences(raw);
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return Err("empty output".to_string());
    }

    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("invalid JSON: {e}"))?;

    let array = match value {
        serde_json::Value::Object(_) => serde_json::Value::Array(vec![value]),
        serde_json::Value::Array(_) => value,
        _ => return Err("expected a JSON object or array".to_string()),
    };

    serde_json::from_value(array).map_err(|e| format!("invalid commit shape: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_array() {
        let raw = "```json\n[{\"message\": \"feat: x\", \"files\": [\"a\"]}]\n```";
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
    fn normalizes_lone_object_to_array() {
        let raw = "{\"message\": \"fix: y\", \"files\": [\"b\", \"c\"]}";
        let got = parse_proposals(raw).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].files, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn parses_fenced_lone_object() {
        let raw = "```\n{\"message\": \"chore: z\", \"files\": [\"d\"]}\n```";
        assert_eq!(parse_proposals(raw).unwrap().len(), 1);
    }

    #[test]
    fn rejects_non_object_or_array() {
        assert!(parse_proposals("\"just a string\"").is_err());
        assert!(parse_proposals("42").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_proposals("").is_err());
        assert!(parse_proposals("```json\n```").is_err());
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(parse_proposals("[{\"message\": \"feat: x\"}]").is_err());
    }

    #[test]
    fn strip_fences_removes_fence_lines_only() {
        assert_eq!(strip_fences("```json\nkeep\n```"), "keep");
        assert_eq!(strip_fences("a\nb"), "a\nb");
    }
}
