//! JSON の合成（`format = "json"`）: トップレベル shallow merge（[`shallow`]）とキー単位で
//! 再帰する deep merge（[`deep`]）。

use serde_json::{Map, Value};

/// `base`（現在の内容）へ JSON 断片 `frag` をトップレベル shallow merge（後勝ち）する。`base = None`
/// なら `frag` だけを合成する。`frag`・`base` は JSON オブジェクトを要する。
///
/// `base` の意味は「dotfiles 非管理のトップレベルキーを全保持し、dotfiles 所有キー（`frag` が
/// 定義するキー）だけを `frag` で上書きする」。出力は pretty JSON ＋末尾改行。
pub fn shallow(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
    let mut merged = Map::new();

    // base（現在の内容）があれば最下層の土台として最初に畳む（非管理キーを保持）。
    if let Some(base) = base {
        for (k, v) in parse_object(base).map_err(|e| format!("base {e}"))? {
            merged.insert(k, v);
        }
    }
    // frag を後勝ちで重ねる。dotfiles 所有キーが土台を上書き（トップレベル粒度）。
    for (k, v) in parse_object(frag)? {
        merged.insert(k, v);
    }

    let mut out = serde_json::to_vec_pretty(&Value::Object(merged))
        .map_err(|e| format!("JSON 直列化失敗: {e}"))?;
    out.push(b'\n');
    Ok(out)
}

/// `base`（現在の内容）へ JSON 断片 `frag` を deep merge する: object はキー単位で再帰マージ（後勝ち）・
/// 配列は `base` → `frag` の順で連結（dedup・位置対応はしない ― 動機の hooks 配列は「断片の追記」が
/// 意図で、キー付き dedup はスキーマ知識をエンジンへ持ち込むため不採用）・スカラおよび型不一致は
/// 後勝ち（`frag` が勝つ）。`base = None` なら `frag` だけを合成する。`frag`・`base` は JSON オブジェクト
/// を要する。
pub fn deep(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
    let frag = parse_object(frag)?;
    let mut merged = match base {
        Some(base) => parse_object(base).map_err(|e| format!("base {e}"))?,
        None => Map::new(),
    };
    deep_merge_object(&mut merged, frag);

    let mut out = serde_json::to_vec_pretty(&Value::Object(merged))
        .map_err(|e| format!("JSON 直列化失敗: {e}"))?;
    out.push(b'\n');
    Ok(out)
}

/// object をキー単位で重ねる（[`deep`] が使う）。既存キーは [`deep_merge_value`] へ再帰し、
/// `base` に無いキーは `frag` の値をそのまま挿入する。
fn deep_merge_object(base: &mut Map<String, Value>, frag: Map<String, Value>) {
    for (k, frag_v) in frag {
        match base.get_mut(&k) {
            Some(base_v) => deep_merge_value(base_v, frag_v),
            None => {
                base.insert(k, frag_v);
            }
        }
    }
}

/// 1 つの値を重ねる: object 同士は [`deep_merge_object`] へ再帰・array 同士は連結、それ以外
/// （スカラ・型不一致）は後勝ちで `frag` が `base` を置き換える。
fn deep_merge_value(base: &mut Value, frag: Value) {
    match (base, frag) {
        (Value::Object(base), Value::Object(frag)) => deep_merge_object(base, frag),
        (Value::Array(base), Value::Array(frag)) => base.extend(frag),
        (base, frag) => *base = frag,
    }
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

    // ── shallow ──

    #[test]
    fn shallow_frag_wins_over_base() {
        // 宣言順・後勝ち: frag が base の同名キーを上書きする。
        let out = shallow(Some(br#"{"a":1,"b":2}"#), br#"{"b":3,"c":4}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 3); // 後勝ち。
        assert_eq!(v["c"], 4);
    }

    #[test]
    fn shallow_base_preserves_unmanaged_and_overwrites_owned() {
        // base＝既存 dst。dotfiles 断片（frag）が定義するキー（language）は frag が勝ち、frag が
        // 定義しない非管理キー（model / effortLevel）は土台のまま全保持される。
        let frag = br#"{"language":"ja","hook":"base"}"#;
        let existing = br#"{"model":"local","effortLevel":"high","language":"en"}"#;
        let out = shallow(Some(existing), frag).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "local"); // 非管理キーは保持。
        assert_eq!(v["effortLevel"], "high"); // 非管理キーは保持。
        assert_eq!(v["language"], "ja"); // dotfiles 所有キーは frag が土台を上書き。
        assert_eq!(v["hook"], "base"); // frag のキーは残る。
    }

    #[test]
    fn shallow_owned_key_replaces_wholesale_not_deep_merge() {
        // dotfiles 所有のトップレベルキーはオブジェクトごと置き換え（deep merge しない）。
        let out = shallow(Some(br#"{"hooks":{"b":2}}"#), br#"{"hooks":{"a":1}}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["hooks"]["a"], 1);
        assert!(
            v["hooks"].get("b").is_none(),
            "トップレベル粒度で丸ごと置換（配下を deep merge しない）"
        );
    }

    #[test]
    fn shallow_without_base_is_frag_only() {
        // base が無ければ純粋に frag だけを合成する（preserve 無しの純 dotfiles 所有 json）。
        let out = shallow(None, br#"{"model":"shared"}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "shared");
    }

    #[test]
    fn shallow_rejects_non_object_fragment_or_base() {
        assert!(shallow(None, b"[1,2,3]").is_err());
        assert!(shallow(None, b"\"scalar\"").is_err());
        assert!(shallow(Some(b"[1,2,3]"), b"{}").is_err());
    }

    // ── deep ──

    #[test]
    fn deep_merges_nested_objects_by_key() {
        // object はキー単位で再帰マージ（shallow はここを丸ごと置換する）。
        let out = deep(
            Some(br#"{"hooks":{"a":1,"b":2}}"#),
            br#"{"hooks":{"b":3,"c":4}}"#,
        )
        .unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["hooks"]["a"], 1, "base 側だけのキーは保持される");
        assert_eq!(v["hooks"]["b"], 3, "同名キーは frag が後勝ち");
        assert_eq!(v["hooks"]["c"], 4, "frag 側だけのキーは追加される");
    }

    #[test]
    fn deep_concatenates_arrays_in_step_order() {
        // 配列は dedup・位置対応なしで base → frag の順に連結する。
        let out = deep(Some(br#"{"list":[1,2]}"#), br#"{"list":[2,3]}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["list"], serde_json::json!([1, 2, 2, 3]));
    }

    #[test]
    fn deep_scalar_or_type_mismatch_last_wins() {
        // スカラ同士・型不一致のいずれも frag が base を置き換える。
        let out = deep(Some(br#"{"a":1}"#), br#"{"a":2}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"], 2, "スカラは後勝ち");

        let out = deep(Some(br#"{"a":{"x":1}}"#), br#"{"a":[1,2]}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            v["a"],
            serde_json::json!([1, 2]),
            "型不一致（object vs array）は frag が後勝ち"
        );
    }

    #[test]
    fn deep_without_base_is_frag_only() {
        let out = deep(None, br#"{"a":{"b":1}}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"]["b"], 1);
    }
}
