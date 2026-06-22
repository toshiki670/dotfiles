//! placeholder 置換（§9.1-4）: 配置ファイル中の `@@name@@` を named value で埋める。
//!
//! `locals` を宣言した単位の materialize 後バイト列に対し、**解決済みの名前→値**だけを置換する
//! （生成方式 copy / generate を問わず形式非依存）。条件分岐・関数を持つ汎用テンプレートは導入
//! しない（named placeholder の単純置換のみ）。未解決名（ストアに値が無い）は `values` に入らない
//! ため `@@name@@` が literal のまま残り、doctor が検出・再 apply で解消する（§9.1 非 TTY 劣化）。

use std::collections::BTreeMap;

/// `@@<name>@@` を `values` の対応値で置換する。`values` に無い名前は触らない（literal 残し）。
///
/// 置換対象は呼び出し側（apply）が解決した「locals 宣言かつ値あり」の集合のみ。`values` が空なら
/// 入力をそのまま返す（注入対象でない単位は呼び出し側が空マップを渡す＝巻き込み防止）。
pub fn substitute(bytes: &[u8], values: &BTreeMap<String, String>) -> Vec<u8> {
    if values.is_empty() {
        return bytes.to_vec();
    }
    // バイト列に対し UTF-8 を仮定せず置換するため、各 placeholder のバイトパターンで replace する。
    // 設定ファイルはテキストだが、形式非依存を保つためバイト走査で実装する。
    let mut out = bytes.to_vec();
    for (name, value) in values {
        let needle = format!("@@{name}@@");
        out = replace_bytes(&out, needle.as_bytes(), value.as_bytes());
    }
    out
}

/// `haystack` 中の `needle` をすべて `replacement` に置換する（バイト単位）。
fn replace_bytes(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    if needle.is_empty() {
        return haystack.to_vec();
    }
    let mut out = Vec::with_capacity(haystack.len());
    let mut i = 0;
    while i < haystack.len() {
        if haystack[i..].starts_with(needle) {
            out.extend_from_slice(replacement);
            i += needle.len();
        } else {
            out.push(haystack[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn substitutes_declared_names() {
        let src = b"[user]\n  email = @@git.email@@\n  name = @@git.name@@\n";
        let out = substitute(src, &map(&[("git.email", "me@x"), ("git.name", "Toshiki")]));
        assert_eq!(out, b"[user]\n  email = me@x\n  name = Toshiki\n".to_vec(),);
    }

    #[test]
    fn leaves_unresolved_placeholder_literal() {
        // 値が無い名前（values に入らない）は @@…@@ のまま残す（非 TTY 劣化・doctor が検出）。
        let src = b"email = @@git.email@@\n";
        let out = substitute(src, &map(&[]));
        assert_eq!(out, src.to_vec());
    }

    #[test]
    fn does_not_touch_undeclared_placeholders() {
        // 宣言外の @@other@@ は置換しない（values に無い）。
        let src = b"a = @@git.email@@\nb = @@other@@\n";
        let out = substitute(src, &map(&[("git.email", "v")]));
        assert_eq!(out, b"a = v\nb = @@other@@\n".to_vec());
    }

    #[test]
    fn replaces_multiple_occurrences() {
        let src = b"@@k@@-@@k@@";
        let out = substitute(src, &map(&[("k", "x")]));
        assert_eq!(out, b"x-x".to_vec());
    }
}
