//! 合成戦略（§5.5）: 複数の断片を 1 つの dst=ファイルへ重ねる純ロジック。
//!
//! - `concat` … テキスト連結（後ろへ連結。境目に改行を 1 つ補う）。generate の
//!   「cmd 出力＋sibling 連結」もこの戦略へ統一する（出力は従来と不変）。
//! - `json_shallow` … JSON のトップレベル shallow merge（後勝ち）。`base`（既存 dst）を
//!   与えると最下層の土台として最初に畳み、dotfiles 所有のトップレベルキーだけを断片で
//!   上書きする。dotfiles が定義しない非管理キーは土台のまま全保持される（旧 `modify_` の
//!   `jq '$local + $forced'` と同値。deep merge はしない）。
//!
//! どちらも副作用のない純関数で、配置（書き込み）は [`crate::core::apply::compose`] が行う。

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

/// JSON 断片をトップレベル shallow merge（宣言順・後勝ち）する。`base`（既存 dst）を与えると
/// 最下層の土台として最初に畳み、その上に断片を重ねる。各断片・`base` は JSON オブジェクトを要する。
///
/// `base` の意味は「dotfiles 非管理のトップレベルキーを全保持し、dotfiles 所有キー（断片が
/// 定義するキー）だけを断片で上書きする」（旧 `$local + $forced`）。`base` が無ければ純粋に
/// 断片だけを合成する。出力は pretty JSON ＋末尾改行。
pub fn json_shallow(frags: &[Vec<u8>], base: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut merged = Map::new();

    // preserve = true のとき、既存 dst を最下層の土台として最初に畳む（非管理キーを保持）。
    if let Some(base) = base {
        let obj = parse_object(base).map_err(|e| format!("既存 dst {e}"))?;
        for (k, v) in obj {
            merged.insert(k, v);
        }
    }

    for (i, frag) in frags.iter().enumerate() {
        let obj = parse_object(frag).map_err(|e| format!("overlay {} {e}", i + 1))?;
        for (k, v) in obj {
            merged.insert(k, v); // 後勝ち。dotfiles 所有キーが土台を上書き（トップレベル粒度）。
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
        let out = json_shallow(&[b(r#"{"a":1,"b":2}"#), b(r#"{"b":3,"c":4}"#)], None).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 3); // 後勝ち。
        assert_eq!(v["c"], 4);
    }

    #[test]
    fn json_shallow_base_preserves_unmanaged_and_overwrites_owned() {
        // base＝既存 dst。dotfiles 断片が定義するキー（language）は断片が勝ち、断片が
        // 定義しない非管理キー（model / effortLevel）は土台のまま全保持される（旧 $local+$forced）。
        let frag = b(r#"{"language":"ja","hook":"base"}"#);
        let existing = b(r#"{"model":"local","effortLevel":"high","language":"en"}"#);
        let out = json_shallow(&[frag], Some(&existing)).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "local"); // 非管理キーは保持。
        assert_eq!(v["effortLevel"], "high"); // 非管理キーは保持。
        assert_eq!(v["language"], "ja"); // dotfiles 所有キーは断片が土台を上書き。
        assert_eq!(v["hook"], "base"); // 断片のキーは残る。
    }

    #[test]
    fn json_shallow_owned_key_replaces_wholesale_not_deep_merge() {
        // dotfiles 所有のトップレベルキーはオブジェクトごと置き換え（deep merge しない）。
        let frag = b(r#"{"hooks":{"a":1}}"#);
        let existing = b(r#"{"hooks":{"b":2}}"#);
        let out = json_shallow(&[frag], Some(&existing)).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["hooks"]["a"], 1);
        assert!(
            v["hooks"].get("b").is_none(),
            "トップレベル粒度で丸ごと置換（配下を deep merge しない）"
        );
    }

    #[test]
    fn json_shallow_without_base_is_frags_only() {
        // base が無ければ純粋に断片だけを合成する（preserve 無しの純 dotfiles 所有 json）。
        let out = json_shallow(&[b(r#"{"model":"shared"}"#)], None).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "shared");
    }

    #[test]
    fn json_shallow_rejects_non_object_fragment_or_base() {
        assert!(json_shallow(&[b("[1,2,3]")], None).is_err());
        assert!(json_shallow(&[b("\"scalar\"")], None).is_err());
        let bad_base = b("[1,2,3]");
        assert!(json_shallow(&[b("{}")], Some(&bad_base)).is_err());
    }
}
