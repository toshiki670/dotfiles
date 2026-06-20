//! 合成戦略（§5.5）: 複数の断片を 1 つの dst=ファイルへ重ねる純ロジック。
//!
//! - `concat` … テキスト連結（後ろへ連結。境目に改行を 1 つ補う）。generate の
//!   「cmd 出力＋sibling 連結」もこの戦略へ統一する（出力は従来と不変）。
//! - `json_shallow` … JSON のトップレベル shallow merge（後勝ち）。`preserve` キーは
//!   既存 dst の値で最後に上書きし、ローカル値を勝たせる（旧 `modify_` の `$local + $forced`
//!   と同じ粒度。deep merge はしない）。
//!
//! どちらも副作用のない純関数で、配置（書き込み）は [`crate::compose`] が行う。

use serde_json::{Map, Value};

/// テキスト断片を順に連結する。境目では、直前が改行で終わっていなければ改行を 1 つ補う。
///
/// 空入力は空、単一断片はそのまま。generate の旧 `append_siblings`（生成物の後ろへ sibling を
/// 行頭で接ぐ）と同じ規則で、cmd 出力を先頭断片に置けば従来出力と一致する。
pub fn concat(frags: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for frag in frags {
        if out.last().is_some_and(|&b| b != b'\n') {
            out.push(b'\n');
        }
        out.extend_from_slice(frag);
    }
    out
}

/// JSON 断片をトップレベル shallow merge（宣言順・後勝ち）し、`preserve` キーだけ既存 dst の
/// 値で最後に上書きする。各断片・既存 dst は JSON オブジェクトであることを要する。
///
/// 出力は pretty JSON ＋末尾改行。`existing` が無い／`preserve` キーが既存に無い場合は温存をしない。
pub fn json_shallow(
    frags: &[Vec<u8>],
    preserve_keys: &[String],
    existing: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut merged = Map::new();
    for (i, frag) in frags.iter().enumerate() {
        let obj = parse_object(frag).map_err(|e| format!("overlay {} {e}", i + 1))?;
        for (k, v) in obj {
            merged.insert(k, v); // 後勝ち（BTreeMap.insert が既存キーを置換）。
        }
    }

    // preserve（既存 dst を読む built-in overlay）は常に最後に重ね、ローカル値を勝たせる。
    if !preserve_keys.is_empty()
        && let Some(existing) = existing
    {
        let obj = parse_object(existing).map_err(|e| format!("既存 dst {e}"))?;
        for key in preserve_keys {
            if let Some(v) = obj.get(key) {
                merged.insert(key.clone(), v.clone());
            }
        }
    }

    let mut out = serde_json::to_vec_pretty(&Value::Object(merged))
        .map_err(|e| format!("JSON 直列化失敗: {e}"))?;
    out.push(b'\n');
    Ok(out)
}

/// バイト列を JSON オブジェクトとしてパースする（オブジェクト以外はエラー）。
fn parse_object(bytes: &[u8]) -> Result<Map<String, Value>, String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|e| format!("の JSON パース失敗: {e}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err("が JSON オブジェクトでない".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    #[test]
    fn concat_empty_is_empty() {
        assert!(concat(&[]).is_empty());
    }

    #[test]
    fn concat_single_is_verbatim() {
        assert_eq!(
            concat(&[b("complete -c foo -f\n")]),
            b("complete -c foo -f\n")
        );
    }

    #[test]
    fn concat_joins_with_newline_when_missing() {
        // 先頭断片が改行で終わらない → 境目に改行を 1 つ補う。
        assert_eq!(concat(&[b("a"), b("b")]), b("a\nb"));
        // 既に改行で終わる → 二重改行にしない。
        assert_eq!(concat(&[b("a\n"), b("b")]), b("a\nb"));
    }

    #[test]
    fn concat_preserves_generate_output() {
        // 既存 E2E（generate 出力不変）と同じ: 生成物＋sibling。
        assert_eq!(
            concat(&[b("GENERATED\n"), b("# CUSTOM\n")]),
            b("GENERATED\n# CUSTOM\n")
        );
    }

    #[test]
    fn json_shallow_later_frag_wins() {
        let out = json_shallow(&[b(r#"{"a":1,"b":2}"#), b(r#"{"b":3,"c":4}"#)], &[], None).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 3); // 後勝ち。
        assert_eq!(v["c"], 4);
    }

    #[test]
    fn json_shallow_preserve_keeps_existing_value() {
        let base = b(r#"{"model":"shared","hook":"x"}"#);
        let existing = b(r#"{"model":"local","other":"y"}"#);
        let out = json_shallow(&[base], &["model".to_string()], Some(&existing)).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "local"); // 既存（ローカル）値が勝つ。
        assert_eq!(v["hook"], "x"); // base のキーは残る。
        assert!(
            v.get("other").is_none(),
            "preserve 外の既存キーは持ち込まない"
        );
    }

    #[test]
    fn json_shallow_preserve_noop_when_key_absent_or_no_existing() {
        // 既存に preserve キーが無い → 温存しない（base 値のまま）。
        let empty = b(r#"{}"#);
        let out = json_shallow(
            &[b(r#"{"model":"shared"}"#)],
            &["model".to_string()],
            Some(&empty),
        )
        .unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "shared");
        // 既存ファイルが無い → エラーにせず base のまま。
        let out2 =
            json_shallow(&[b(r#"{"model":"shared"}"#)], &["model".to_string()], None).unwrap();
        let v2: Value = serde_json::from_slice(&out2).unwrap();
        assert_eq!(v2["model"], "shared");
    }

    #[test]
    fn json_shallow_rejects_non_object_fragment() {
        assert!(json_shallow(&[b("[1,2,3]")], &[], None).is_err());
        assert!(json_shallow(&[b("\"scalar\"")], &[], None).is_err());
    }
}
