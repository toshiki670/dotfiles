//! `dotfiles doctor`（雛形・§9.5・§11）: `locals` 宣言がストアに揃っているかを診断する。
//!
//! 全単位の `manifest.toml`（[`crate::core::discover`]）から `locals` を集め、ストア（[`crate::core::locals::store`]）に
//! 値が無い名前を報告する。ツール別ロジックは持たず「宣言名がストアに在るか」を見るだけ（§9.5。
//! git の `git config --includes` 解決スコープに依存しない）。雛形のため依存バイナリ等の他診断は
//! 後続スライス。未設定があっても**ブロックしない**（exit 0・情報提供）。

use crate::core::discover::{self, MANIFEST};
use crate::core::locals::store::Store;
use crate::core::manifest::Manifest;
use std::path::Path;

/// `source` 配下の `locals` 宣言とストアを突き合わせ、未設定を stderr に報告する。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    let store = Store::load(home)?;

    let mut missing = Vec::new();
    for unit in &units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        for name in &manifest.locals {
            if store.get(name).is_none() {
                missing.push((unit.rel.to_string_lossy().into_owned(), name.clone()));
            }
        }
    }

    if missing.is_empty() {
        println!("doctor: locals は全て設定済み");
        return Ok(());
    }
    eprintln!("doctor: 未設定の locals が {} 件あります:", missing.len());
    for (unit, name) in &missing {
        eprintln!("  - {name}（{unit}）: `dotfiles local set {name} <value>` で設定");
    }
    Ok(())
}
