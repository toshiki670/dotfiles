//! `--explain` 用の `claude -p` 要約呼び出し。
//!
//! gcm の `claude.rs`/`proposals.rs` と同じ envelope デコードパターンを使うが、bin 間で
//! 共有する lib が無い設計のため、この bin 専用に再実装する。`outdated --explain` は
//! 対話しない one-shot 要約なので、gcm にある修正指示トリガーの sonnet フォールバックは
//! 持たず、モデルは `haiku` 固定にする。

use std::io::Write;
use std::process::{Command, Stdio};

use serde::Deserialize;

const SYSTEM_PROMPT: &str = "You are a release notes summarizer.

Summarize the given release notes in Japanese: what's new, what's fixed. Keep it concise (a few sentences).";

/// 構造化出力を強制するスキーマ。
const OUTPUT_SCHEMA: &str =
    r#"{"type":"object","properties":{"summary":{"type":"string"}},"required":["summary"]}"#;

/// リリースノート本文を claude -p で日本語要約する。
///
/// `claude` の起動失敗・空出力・パース失敗はすべて警告して `None` を返す
/// （呼び出し元は要約なしで続行する）。
pub fn summarize(release_notes: &str) -> Option<String> {
    let mut child = match Command::new("claude")
        .args([
            "-p",
            "--model",
            "haiku",
            "--system-prompt",
            SYSTEM_PROMPT,
            "--json-schema",
            OUTPUT_SCHEMA,
            "--output-format",
            "json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!(
                "⚠️  要約の生成に失敗しました。claude コマンドが利用可能か確認してください。"
            );
            return None;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(release_notes.as_bytes());
        // stdin はここで drop され閉じる。
    }

    let output = child.wait_with_output().ok()?;
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    if raw.trim().is_empty() {
        eprintln!("⚠️  要約の生成に失敗しました。claude コマンドが利用可能か確認してください。");
        return None;
    }

    match parse_summary(&raw) {
        Ok(summary) => Some(summary),
        Err(msg) => {
            eprintln!("⚠️  要約の生成に失敗しました: {msg}");
            None
        }
    }
}

/// `--output-format json` の結果 envelope。`summary` フィールドだけ拾う。
#[derive(Deserialize)]
struct Envelope {
    is_error: bool,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    structured_output: Option<StructuredOutput>,
}

#[derive(Deserialize)]
struct StructuredOutput {
    summary: String,
}

/// claude の envelope から要約テキストを取り出す。
fn parse_summary(raw: &str) -> Result<String, String> {
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
        .map(|output| output.summary)
        .ok_or_else(|| "missing structured_output".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_envelope() {
        let raw = r#"{"type":"result","is_error":false,"structured_output":{"summary":"新機能Xを追加、バグYを修正"}}"#;
        assert_eq!(parse_summary(raw).unwrap(), "新機能Xを追加、バグYを修正");
    }

    #[test]
    fn error_envelope_returns_error_message() {
        let raw = r#"{"is_error":true,"errors":["Reached maximum budget ($0.0001)"]}"#;
        let err = parse_summary(raw).unwrap_err();
        assert!(err.contains("budget"));
    }

    #[test]
    fn missing_structured_output_is_error() {
        assert!(parse_summary(r#"{"is_error":false}"#).is_err());
    }

    #[test]
    fn malformed_structured_output_is_error() {
        assert!(parse_summary(r#"{"is_error":false,"structured_output":42}"#).is_err());
    }

    #[test]
    fn invalid_json_is_error() {
        assert!(parse_summary("not json at all").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_summary("").is_err());
    }
}
