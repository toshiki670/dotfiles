//! plist の合成（`format = "plist"`）: [`super::json`] と同じ shallow / deep merge を、plist の
//! dict モデルへ適用する。

use plist::{Dictionary, Value as PlistValue};
use std::io::Cursor;

/// `base`（現在の内容）へ plist 断片 `frag` をトップレベル shallow merge（後勝ち）する。
/// [`super::json::shallow`] の plist 版。`base = None` なら `frag` だけを合成する。`frag`・`base`
/// は plist 辞書（トップレベル dict）を要し、`parse_dict`（`plist::Value::from_reader`）が
/// XML/binary/ASCII のどの直列化でも自動判別する。出力は XML plist に固定する（差分可読性。#465）。
///
/// 生きたドメインの export を土台に、リポジトリ管理の断片を dict キー単位で上書きする用途を想定する
/// （どの引数へ何を渡すかの運用は呼び出し元 [`crate::apply::pipeline`] の責務。本関数はマージのみ）。
pub fn shallow(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
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

/// `base`（現在の内容）へ plist 断片 `frag` を deep merge する。[`super::json::deep`] の plist 版
/// （dict はキー単位で再帰マージ・array は連結・スカラおよび型不一致は後勝ち）。`base = None` なら
/// `frag` だけを合成する。`frag`・`base` は plist 辞書（トップレベル dict）を要する。
pub fn deep(base: Option<&[u8]>, frag: &[u8]) -> Result<Vec<u8>, String> {
    let frag = parse_dict(frag)?;
    let mut merged = match base {
        Some(base) => parse_dict(base).map_err(|e| format!("base {e}"))?,
        None => Dictionary::new(),
    };
    deep_merge_dict(&mut merged, frag);

    let mut out = Vec::new();
    PlistValue::Dictionary(merged)
        .to_writer_xml(&mut out)
        .map_err(|e| format!("plist 直列化失敗: {e}"))?;
    out.push(b'\n');
    Ok(out)
}

/// dict をキー単位で重ねる（[`deep`] が使う）。
fn deep_merge_dict(base: &mut Dictionary, frag: Dictionary) {
    for (k, frag_v) in frag {
        match base.get_mut(&k) {
            Some(base_v) => deep_merge_plist_value(base_v, frag_v),
            None => {
                base.insert(k, frag_v);
            }
        }
    }
}

/// 1 つの値を重ねる: dict 同士は [`deep_merge_dict`] へ再帰・array 同士は連結、それ以外は後勝ち。
fn deep_merge_plist_value(base: &mut PlistValue, frag: PlistValue) {
    match (base, frag) {
        (PlistValue::Dictionary(base), PlistValue::Dictionary(frag)) => {
            deep_merge_dict(base, frag);
        }
        (PlistValue::Array(base), PlistValue::Array(frag)) => base.extend(frag),
        (base, frag) => *base = frag,
    }
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

    /// `<dict>…</dict>` の中身だけを渡して XML plist 1 枚に包む（テスト用の短縮ヘルパ）。
    fn plist_dict(body: &str) -> Vec<u8> {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>{body}</dict></plist>"
        )
        .into_bytes()
    }

    /// shallow / deep の出力を plist 辞書へ戻す（アサーション用。トップレベル dict 以外は panic）。
    fn dict_of(bytes: &[u8]) -> Dictionary {
        match PlistValue::from_reader(Cursor::new(bytes)).unwrap() {
            PlistValue::Dictionary(d) => d,
            other => panic!("トップレベルが dict でない: {other:?}"),
        }
    }

    // ── shallow ──

    #[test]
    fn shallow_frag_wins_over_base() {
        let base = plist_dict("<key>a</key><integer>1</integer><key>b</key><integer>2</integer>");
        let frag = plist_dict("<key>b</key><integer>3</integer><key>c</key><integer>4</integer>");
        let out = shallow(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["a"].as_signed_integer(), Some(1));
        assert_eq!(d["b"].as_signed_integer(), Some(3)); // 後勝ち。
        assert_eq!(d["c"].as_signed_integer(), Some(4));
    }

    #[test]
    fn shallow_base_preserves_unmanaged_and_overwrites_owned() {
        // base＝生きたドメインの `defaults export` 相当（Window Frame 等のローカル状態を含む）。
        // dotfiles 断片（frag）が定義するキー（CPU_state）は frag が勝ち、frag が定義しない非管理キー
        // （NSWindow Frame）は土台のまま全保持される（#531 の反映フローと同じ形）。
        let base = plist_dict(
            "<key>NSWindow Frame</key><string>0 0 100 200</string>\
             <key>CPU_state</key><false/>",
        );
        let frag = plist_dict("<key>CPU_state</key><true/>");
        let out = shallow(Some(&base), &frag).unwrap();
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
    fn shallow_owned_key_replaces_wholesale_not_deep_merge() {
        // dotfiles 所有のトップレベルキーはオブジェクトごと置き換え（deep merge しない）。
        let base = plist_dict("<key>toolbar</key><dict><key>b</key><integer>2</integer></dict>");
        let frag = plist_dict("<key>toolbar</key><dict><key>a</key><integer>1</integer></dict>");
        let out = shallow(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        let toolbar = d["toolbar"].as_dictionary().unwrap();
        assert_eq!(toolbar["a"].as_signed_integer(), Some(1));
        assert!(
            toolbar.get("b").is_none(),
            "トップレベル粒度で丸ごと置換（配下を deep merge しない）"
        );
    }

    #[test]
    fn shallow_without_base_is_frag_only() {
        // base が無ければ純粋に frag だけを合成する（初回 apply・生きたドメイン未作成時と同じ形）。
        let out = shallow(None, &plist_dict("<key>k</key><string>v</string>")).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["k"].as_string(), Some("v"));
    }

    #[test]
    fn shallow_empty_dict_frag_stays_empty() {
        // ドメイン未作成時の `defaults export` が返す空 dict を最初の input（base 無し）にしても
        // 空 dict へ安全に畳める。
        let out = shallow(None, &plist_dict("")).unwrap();
        let d = dict_of(&out);
        assert!(d.is_empty());
    }

    #[test]
    fn shallow_rejects_non_dict_fragment_or_base() {
        let array = b"<?xml version=\"1.0\"?><plist version=\"1.0\"><array/></plist>".to_vec();
        assert!(shallow(None, &array).is_err());
        assert!(shallow(Some(&array), &plist_dict("")).is_err());
    }

    #[test]
    fn shallow_accepts_binary_plist_input_not_just_xml() {
        // plist はデータモデルであり「XML」はその直列化の1つにすぎない（#531）。base を binary
        // plist（`defaults export <domain> <file>` が実際に書く形式）で与えても、xml plist の断片と
        // 同じく shallow merge できることを確認する ― `shallow` が「XML の合成」ではなく
        // 「plist（直列化非依存）の合成」であることの直接の証拠。
        let base_dict = plist::plist_dict! {
            "NSWindow Frame": "0 0 100 200",
            "CPU_state": false,
        };
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
        let out = shallow(Some(&base_bin), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["NSWindow Frame"].as_string(), Some("0 0 100 200"));
        assert_eq!(d["CPU_state"].as_boolean(), Some(true)); // frag が binary の土台を上書き。
    }

    // ── deep ──

    #[test]
    fn deep_merges_nested_dicts_by_key() {
        let base = plist_dict(
            "<key>toolbar</key><dict><key>a</key><integer>1</integer><key>b</key><integer>2</integer></dict>",
        );
        let frag = plist_dict(
            "<key>toolbar</key><dict><key>b</key><integer>3</integer><key>c</key><integer>4</integer></dict>",
        );
        let out = deep(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        let toolbar = d["toolbar"].as_dictionary().unwrap();
        assert_eq!(
            toolbar["a"].as_signed_integer(),
            Some(1),
            "base 側キーは保持"
        );
        assert_eq!(
            toolbar["b"].as_signed_integer(),
            Some(3),
            "同名キーは後勝ち"
        );
        assert_eq!(
            toolbar["c"].as_signed_integer(),
            Some(4),
            "frag 側キーは追加"
        );
    }

    #[test]
    fn deep_concatenates_arrays_in_step_order() {
        let base = plist_dict("<key>list</key><array><integer>1</integer></array>");
        let frag = plist_dict("<key>list</key><array><integer>2</integer></array>");
        let out = deep(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        let list = d["list"].as_array().unwrap();
        assert_eq!(
            list.iter()
                .map(|v| v.as_signed_integer().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2],
            "配列は base → frag の順で連結"
        );
    }

    #[test]
    fn deep_scalar_last_wins() {
        let base = plist_dict("<key>CPU_state</key><false/>");
        let frag = plist_dict("<key>CPU_state</key><true/>");
        let out = deep(Some(&base), &frag).unwrap();
        let d = dict_of(&out);
        assert_eq!(d["CPU_state"].as_boolean(), Some(true));
    }
}
