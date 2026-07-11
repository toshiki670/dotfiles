//! テキストの合成（`format = "text"` ＋ `merge = "append"`）: 断片を後ろへ連結する。

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
