//! named value の注入（§9.1 step4）: 配置ファイル中の `@@name@@` をストア値で置換する。
//!
//! 生成方式（copy / generate）を問わず、`locals` を宣言したユニットの配置ファイル（[`crate::copy`]
//! / [`crate::compose`] が書き出した実ファイル）に対し materialize 後に走る。置換は **named
//! placeholder のみ**で、条件分岐・関数を持つ汎用テンプレートは導入しない（設計書 §9.1）。
//!
//! 解決済みの名前だけを置換し、**未解決（ストアに無い）の `@@name@@` はリテラルのまま残す**。
//! 空置換せず可視に残すことで `doctor` が拾え、誤って空値を焼き込まない。バイト列で扱い、
//! 内容が変わったファイルだけ書き戻す。

use std::collections::BTreeMap;
use std::path::Path;

/// `paths` の各ファイルについて、`values`（名前 → 値）の `@@name@@` を値へ置換する。
/// `values` に無い名前のプレースホルダは触らない。内容が変わったファイルだけ書き戻す。
pub fn substitute(paths: &[&Path], values: &BTreeMap<String, String>) -> Result<(), String> {
    if values.is_empty() {
        return Ok(());
    }
    for path in paths {
        let original =
            std::fs::read(path).map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))?;
        let mut content = original.clone();
        for (name, value) in values {
            let needle = format!("@@{name}@@");
            content = replace_bytes(&content, needle.as_bytes(), value.as_bytes());
        }
        if content != original {
            std::fs::write(path, &content)
                .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// `input` 中の `from` を全て `to` に置換したバイト列を返す（非 UTF-8 でも安全）。
fn replace_bytes(input: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    if from.is_empty() {
        return input.to_vec();
    }
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(from) {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(input[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn replace_bytes_replaces_all_occurrences() {
        assert_eq!(
            replace_bytes(b"a@@x@@b@@x@@", b"@@x@@", b"V"),
            b"aVbV".to_vec()
        );
    }

    #[test]
    fn substitute_replaces_known_and_keeps_unknown_literal() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("user");
        std::fs::write(
            &f,
            "[user]\n\temail = @@git.email@@\n\tname = @@git.name@@\n",
        )
        .unwrap();

        // git.name は与えない → リテラルのまま残るべき。
        substitute(&[&f], &values(&[("git.email", "me@x")])).unwrap();

        let out = std::fs::read_to_string(&f).unwrap();
        assert!(
            out.contains("email = me@x"),
            "解決値が置換されるべき:\n{out}"
        );
        assert!(
            out.contains("name = @@git.name@@"),
            "未解決の placeholder はリテラルのまま残るべき:\n{out}"
        );
    }

    #[test]
    fn substitute_empty_values_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("user");
        std::fs::write(&f, "email = @@git.email@@\n").unwrap();
        substitute(&[&f], &values(&[])).unwrap();
        assert_eq!(
            std::fs::read_to_string(&f).unwrap(),
            "email = @@git.email@@\n"
        );
    }
}
