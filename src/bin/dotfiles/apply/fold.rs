//! 畳み込み: 現在の内容（`base`）へ新しい断片（`frag`）を 1 つ後勝ちで重ねる純ロジック。
//! [`crate::apply::pipeline`] の `fold_in` が step ごとに、現在の内容を `base`・新しい input を
//! `frag` として呼ぶ（`format` が戦略を選ぶ）。
//!
//! - `concat`（`format = "text"`）… テキスト連結（`frag` を後ろへ連結。境目に改行を 1 つ補う）。
//! - `json_shallow`（`format = "json"`）… JSON のトップレベル shallow merge（後勝ち）。`base`
//!   （現在の内容）を最下層の土台とし、dotfiles 所有のトップレベルキーだけを `frag` で上書きする。
//!   dotfiles が定義しない非管理キーは土台のまま全保持される（deep merge はしない）。
//! - `plist_shallow`（`format = "plist"`）… `json_shallow` の plist 版（トップレベル shallow merge・
//!   後勝ち・deep merge しない）。shallow merge を保証するのは plist の dict モデルであって XML という
//!   構文ではないため、`xml_shallow` ではなく `plist_shallow` と呼ぶ。
//!
//! いずれも副作用のない純関数で、配置（書き込み）は [`crate::apply::pipeline`] が行う。`base = None`
//! で呼ぶと `frag` そのもの（再直列化）を返す ― これにより step 列の最初の input（土台なし）と
//! 2 つ目以降（土台あり）を同じ関数で畳める（[`crate::apply::pipeline`] の `fold_in`）。パースエラーの
//! ラベル付け（どの input か）は呼び出し元が担い、本モジュールは `base` 由来のエラーだけ `base` と
//! 前置する（`frag` 由来はそのまま返す）。

use plist::{Dictionary, Value as PlistValue};
use serde_json::{Map, Value};
use std::io::Cursor;

/// `base`（現在の内容）へテキスト断片 `frag` を連結する。境目では、`base` が改行で終わっていなければ
/// 改行を 1 つ補う。`base = None`（土台なし）は `frag` をそのまま返す。
pub fn concat(base: Option<&[u8]>, frag: &[u8]) -> Vec<u8> {
    let mut out = base.map(<[u8]>::to_vec).unwrap_or_default();
    if out.last().is_some_and(|&b| b != b'\n') {
        out.push(b'\n');
    }
    out.extend_from_slice(frag);
    out
}

/// `base`（現在の内容）へ JSON 断片 `frag` をトップレベル shallow merge（後勝ち）する。`base = None`
/// なら `frag` だけを合成する。`frag`・`base` は JSON オブジェクトを要する。
///
/// `base` の意味は「dotfiles 非管理のトップレベルキーを全保持し、dotfiles 所有キー（`frag` が
/// 定義するキー）だけを `frag` で上書きする」。出力は pretty JSON ＋末尾改行。
pub fn json_shallow(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
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

/// バイト列を JSON オブジェクトとしてパースする（オブジェクト以外はエラー）。
fn parse_object(bytes: &[u8]) -> Result<Map<String, Value>, String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|e| format!("の JSON パース失敗: {e}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err("が JSON オブジェクトでない".to_string()),
    }
}

/// `base`（現在の内容）へ plist 断片 `frag` をトップレベル shallow merge（後勝ち）する。`json_shallow`
/// の plist 版。`base = None` なら `frag` だけを合成する。`frag`・`base` は plist 辞書（トップレベル
/// dict）を要し、`parse_dict`（`plist::Value::from_reader`）が XML/binary/ASCII のどの直列化でも
/// 自動判別する。出力は XML plist に固定する（差分可読性。#465）。
///
/// 生きたドメインの export を土台に、リポジトリ管理の断片を dict キー単位で上書きする用途を想定する
/// （どの引数へ何を渡すかの運用は呼び出し元 [`crate::apply::pipeline`] の責務。本関数はマージのみ）。
pub fn plist_shallow(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
    let mut merged = Dictionary::new();

    if let Some(base) = base {
        for (k, v) in parse_dict(base).map_err(|e| format!("base {e}"))? {
            merged.insert(k, v);
        }
    }
    for (k, v) in parse_dict(frag)? {
        merged.insert(k, v); // 後勝ち。dotfiles 所有キーが土台を上書き（トップレベル粒度）。
    }

    let mut out = Vec::new();
    PlistValue::Dictionary(merged)
        .to_writer_xml(&mut out)
        .map_err(|e| format!("plist 直列化失敗: {e}"))?;
    out.push(b'\n');
    Ok(out)
}

/// バイト列を plist 辞書（トップレベル dict）としてパースする（辞書以外・パース不能はエラー）。
/// `Value::from_reader` は XML/binary/ASCII plist を自動判別する。
fn parse_dict(bytes: &[u8]) -> Result<Dictionary, String> {
    let value = PlistValue::from_reader(Cursor::new(bytes))
        .map_err(|e| format!("の plist パース失敗: {e}"))?;
    match value {
        PlistValue::Dictionary(dict) => Ok(dict),
        _ => Err("が plist 辞書（トップレベル dict）でない".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── concat（text append） ──

    #[test]
    fn concat_first_frag_is_verbatim() {
        // base 無し（最初の input）は断片そのまま。
        assert_eq!(
            concat(None, b"complete -c foo -f\n"),
            b"complete -c foo -f\n"
        );
    }

    #[test]
    fn concat_joins_base_and_frag_with_single_newline() {
        // base が改行で終わらない → 境目に改行を 1 つ補う。
        assert_eq!(concat(Some(b"a"), b"b"), b"a\nb");
        // base が既に改行で終わる → 二重改行にしない。
        assert_eq!(concat(Some(b"a\n"), b"b"), b"a\nb");
    }

    #[test]
    fn concat_preserves_generate_output() {
        // 既存 E2E（generate 出力不変）と同じ: 生成物（base）＋sibling（frag）。
        assert_eq!(
            concat(Some(b"GENERATED\n"), b"# CUSTOM\n"),
            b"GENERATED\n# CUSTOM\n"
        );
    }

    // ── json_shallow ──

    #[test]
    fn json_shallow_frag_wins_over_base() {
        // 宣言順・後勝ち: frag が base の同名キーを上書きする。
        let out = json_shallow(Some(br#"{"a":1,"b":2}"#), br#"{"b":3,"c":4}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 3); // 後勝ち。
        assert_eq!(v["c"], 4);
    }

    #[test]
    fn json_shallow_base_preserves_unmanaged_and_overwrites_owned() {
        // base＝既存 dst。dotfiles 断片（frag）が定義するキー（language）は frag が勝ち、frag が
        // 定義しない非管理キー（model / effortLevel）は土台のまま全保持される。
        let frag = br#"{"language":"ja","hook":"base"}"#;
        let existing = br#"{"model":"local","effortLevel":"high","language":"en"}"#;
        let out = json_shallow(Some(existing), frag).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "local"); // 非管理キーは保持。
        assert_eq!(v["effortLevel"], "high"); // 非管理キーは保持。
        assert_eq!(v["language"], "ja"); // dotfiles 所有キーは frag が土台を上書き。
        assert_eq!(v["hook"], "base"); // frag のキーは残る。
    }

    #[test]
    fn json_shallow_owned_key_replaces_wholesale_not_deep_merge() {
        // dotfiles 所有のトップレベルキーはオブジェクトごと置き換え（deep merge しない）。
        let out = json_shallow(Some(br#"{"hooks":{"b":2}}"#), br#"{"hooks":{"a":1}}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["hooks"]["a"], 1);
        assert!(
            v["hooks"].get("b").is_none(),
            "トップレベル粒度で丸ごと置換（配下を deep merge しない）"
        );
    }

    #[test]
    fn json_shallow_without_base_is_frag_only() {
        // base が無ければ純粋に frag だけを合成する（preserve 無しの純 dotfiles 所有 json）。
        let out = json_shallow(None, br#"{"model":"shared"}"#).unwrap();
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["model"], "shared");
    }

    #[test]
    fn json_shallow_rejects_non_object_fragment_or_base() {
        assert!(json_shallow(None, b"[1,2,3]").is_err());
        assert!(json_shallow(None, b"\"scalar\"").is_err());
        assert!(json_shallow(Some(b"[1,2,3]"), b"{}").is_err());
    }

    // ── plist_shallow ──

    /// `<dict>…</dict>` の中身だけを渡して XML plist 1 枚に包む（テスト用の短縮ヘルパ）。
    fn plist_dict(body: &str) -> Vec<u8> {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>{body}</dict></plist>"
        )
        .into_bytes()
    }

    /// plist_shallow の出力を plist 辞書へ戻す（アサーション用。トップレベル dict 以外は panic）。
    fn dict_of(bytes: &[u8]) -> Dictionary {
        match PlistValue::from_reader(Cursor::new(bytes)).unwrap() {
            PlistValue::Dictionary(d) => d,
            other => panic!("トップレベルが dict でない: {other:?}"),
        }
    }

    #[test]
    fn plist_shallow_frag_wins_over_base() {
        let base = plist_dict("<key>a</key><integer>1</integer><key>b</key><integer>2</integer>");
        let frag = plist_dict("<key>b</key><integer>3</integer><key>c</key><integer>4</integer>");
        let out = plist_shallow(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["a"].as_signed_integer(), Some(1));
        assert_eq!(d["b"].as_signed_integer(), Some(3)); // 後勝ち。
        assert_eq!(d["c"].as_signed_integer(), Some(4));
    }

    #[test]
    fn plist_shallow_base_preserves_unmanaged_and_overwrites_owned() {
        // base＝生きたドメインの `defaults export` 相当（Window Frame 等のローカル状態を含む）。
        // dotfiles 断片（frag）が定義するキー（CPU_state）は frag が勝ち、frag が定義しない非管理キー
        // （NSWindow Frame）は土台のまま全保持される（#531 の反映フローと同じ形）。
        let base = plist_dict(
            "<key>NSWindow Frame</key><string>0 0 100 200</string>\
             <key>CPU_state</key><false/>",
        );
        let frag = plist_dict("<key>CPU_state</key><true/>");
        let out = plist_shallow(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(
            d["NSWindow Frame"].as_string(),
            Some("0 0 100 200"),
            "非管理キーはローカル状態のまま保持される"
        );
        assert_eq!(
            d["CPU_state"].as_boolean(),
            Some(true),
            "dotfiles 所有キーは frag が土台を上書き"
        );
    }

    #[test]
    fn plist_shallow_owned_key_replaces_wholesale_not_deep_merge() {
        // dotfiles 所有のトップレベルキーはオブジェクトごと置き換え（deep merge しない）。
        // json_shallow_owned_key_replaces_wholesale_not_deep_merge の plist 版。
        let base = plist_dict("<key>toolbar</key><dict><key>b</key><integer>2</integer></dict>");
        let frag = plist_dict("<key>toolbar</key><dict><key>a</key><integer>1</integer></dict>");
        let out = plist_shallow(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        let toolbar = d["toolbar"].as_dictionary().unwrap();
        assert_eq!(toolbar["a"].as_signed_integer(), Some(1));
        assert!(
            toolbar.get("b").is_none(),
            "トップレベル粒度で丸ごと置換（配下を deep merge しない）"
        );
    }

    #[test]
    fn plist_shallow_without_base_is_frag_only() {
        // base が無ければ純粋に frag だけを合成する（初回 apply・生きたドメイン未作成時と同じ形）。
        let out = plist_shallow(None, &plist_dict("<key>k</key><string>v</string>")).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["k"].as_string(), Some("v"));
    }

    #[test]
    fn plist_shallow_empty_dict_frag_stays_empty() {
        // ドメイン未作成時の `defaults export` が返す空 dict を最初の input（base 無し）にしても
        // 空 dict へ安全に畳める。
        let out = plist_shallow(None, &plist_dict("")).unwrap();
        let d = dict_of(&out);
        assert!(d.is_empty());
    }

    #[test]
    fn plist_shallow_rejects_non_dict_fragment_or_base() {
        let array = b"<?xml version=\"1.0\"?><plist version=\"1.0\"><array/></plist>".to_vec();
        assert!(plist_shallow(None, &array).is_err());
        assert!(plist_shallow(Some(&array), &plist_dict("")).is_err());
    }

    #[test]
    fn plist_shallow_accepts_binary_plist_input_not_just_xml() {
        // plist はデータモデルであり「XML」はその直列化の1つにすぎない（#531）。base を binary
        // plist（`defaults export <domain> <file>` が実際に書く形式）で与えても、xml plist の断片と
        // 同じく shallow merge できることを確認する ― `plist_shallow` が「XML の合成」ではなく
        // 「plist（直列化非依存）の合成」であることの直接の証拠。
        let mut base_dict = Dictionary::new();
        base_dict.insert(
            "NSWindow Frame".to_string(),
            PlistValue::String("0 0 100 200".to_string()),
        );
        base_dict.insert("CPU_state".to_string(), PlistValue::Boolean(false));
        let mut base_bin = Vec::new();
        PlistValue::Dictionary(base_dict)
            .to_writer_binary(&mut base_bin)
            .unwrap();
        assert_eq!(
            &base_bin[..8],
            b"bplist00",
            "テストの前提: base が本当に binary plist であること"
        );

        let frag = plist_dict("<key>CPU_state</key><true/>");
        let out = plist_shallow(Some(&base_bin), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["NSWindow Frame"].as_string(), Some("0 0 100 200"));
        assert_eq!(d["CPU_state"].as_boolean(), Some(true)); // frag が binary の土台を上書き。
    }
}
