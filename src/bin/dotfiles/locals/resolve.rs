//! apply 時のローカル値解決: 単位の `locals` をストアと突き合わせ、未設定値を
//! TTY なら対話取得、非 TTY なら警告のみで継続する。
//!
//! ここで解決できた「名前→値」だけを [`crate::locals::inject`] が placeholder 置換に使う。未解決の名前は
//! マップに入らないため、配置ファイルでは `@@name@@` が literal のまま残り doctor が検出する。

use super::store::Store;
use super::{inject, prompt};
use crate::manifest::Manifest;
use std::collections::BTreeMap;

/// 単位の `locals` を解決し、注入用の「名前→値」マップ（解決できた名前のみ）を返す。
///
/// - ストアに既にあれば採用。
/// - 無く `interactive`（TTY）なら [`prompt::ask`] で取得しストアへ 0600 で書く。
/// - 無く非 TTY なら警告（stderr）のみで継続し、その名前はマップに含めない（literal 残し）。
///
/// `manifest.locals` が空なら空マップ（注入対象でない単位）。空マップは [`inject::substitute`] が
/// 素通しするため、`locals` を宣言しない単位のファイルは一切走査されない。
pub fn fill(
    manifest: &Manifest,
    store: &mut Store,
    interactive: bool,
) -> Result<BTreeMap<String, String>, String> {
    let mut resolved = BTreeMap::new();
    for name in &manifest.locals {
        if let Some(value) = store.get(name) {
            resolved.insert(name.clone(), value.to_string());
            continue;
        }
        if interactive {
            let value = prompt::ask(name)?;
            store.set(name, &value);
            store.save()?;
            resolved.insert(name.clone(), value);
        } else {
            eprintln!(
                "apply: locals `{name}` 未設定（`dotfiles local set {name} <value>` で設定）"
            );
        }
    }
    Ok(resolved)
}

/// バイト列へ解決済みローカル値を注入する薄いラッパ（[`inject::substitute`] の再公開）。
/// copy / pipeline の両配置経路（[`crate::apply::copy`] / [`crate::apply::pipeline`]）が同じ注入を
/// 通すための単一窓口。
pub fn inject(bytes: &[u8], values: &BTreeMap<String, String>) -> Vec<u8> {
    inject::substitute(bytes, values)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テキストから Manifest を読み込む（パース＋解釈）。
    fn manifest(src: &str) -> Manifest {
        src.parse().unwrap()
    }

    #[test]
    fn non_interactive_uses_store_values_and_warns_missing() {
        let home = tempfile::tempdir().unwrap();
        let mut store = Store::load(home.path()).unwrap();
        store.set("git.email", "me@example.com");

        let m = manifest(
            "locals = [\"git.email\", \"git.name\"]\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n",
        );
        // 非 TTY: git.email はストアから解決、git.name は未設定で警告のみ（マップに入らない）。
        let resolved = fill(&m, &mut store, false).unwrap();
        assert_eq!(
            resolved.get("git.email").map(String::as_str),
            Some("me@example.com")
        );
        assert_eq!(resolved.get("git.name"), None);
    }

    #[test]
    fn empty_locals_yields_empty_map() {
        let home = tempfile::tempdir().unwrap();
        let mut store = Store::load(home.path()).unwrap();
        let m = manifest("[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n");
        assert!(fill(&m, &mut store, false).unwrap().is_empty());
    }
}
