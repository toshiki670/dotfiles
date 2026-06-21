//! `dotfiles doctor`（雛形）: マシンローカル値（`locals`）の未設定を診断する（§9.1 step5）。
//!
//! 全 unit の `locals` 宣言を集約し、ストア（[`crate::local_store`]）に在るかを見るだけ。ツール別
//! ロジック（git の `git config --includes` 解決スコープ等）に依存しない（設計書 §9.2）。本スライス
//! では locals の set/unset 診断に限る（依存バイナリ等の他チェックは後続）。値は一切表示しない。

use crate::discover::{self, MANIFEST};
use crate::local_store;
use crate::manifest::Manifest;
use std::collections::BTreeMap;
use std::path::Path;

/// 1 つの named value 宣言の集約（複数 unit が同名を宣言しうる）。
struct Decl {
    /// 宣言したユニット名（source 相対）。表示用。
    units: Vec<String>,
    /// いずれかの unit が秘匿指定したか（表示で `hidden` を添える。値は出さない）。
    sensitive: bool,
}

/// `source` 配下の `locals` を集約し、ストア照合で set/unset を診断する。
/// 未設定があっても **警告のみ・exit 0**（automation をブロックしない）。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    let mut decls: BTreeMap<String, Decl> = BTreeMap::new();
    for unit in &units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        let name = unit.rel.to_string_lossy().into_owned();
        for local in &manifest.locals {
            let sensitive = manifest.sensitive.iter().any(|s| s == local);
            let decl = decls.entry(local.clone()).or_insert_with(|| Decl {
                units: Vec::new(),
                sensitive: false,
            });
            decl.units.push(name.clone());
            decl.sensitive |= sensitive;
        }
    }

    let store = local_store::load(home)?;
    println!(
        "dotfiles doctor（source: {}, store: {}）",
        source.display(),
        local_store::path(home).display(),
    );
    if decls.is_empty() {
        println!("  locals 宣言なし");
        return Ok(());
    }

    let mut unset = 0;
    for (name, decl) in &decls {
        let from = decl.units.join(", ");
        let tag = if decl.sensitive { " (sensitive)" } else { "" };
        if store.contains_key(name) {
            println!("  ✓ {name}{tag}  — set  [{from}]");
        } else {
            unset += 1;
            println!("  ⚠ {name}{tag}  — 未設定  [{from}]");
        }
    }
    if unset > 0 {
        println!(
            "locals: {total} 件中 {unset} 件が未設定。`dotfiles secret set <name> <value>` で設定できます。",
            total = decls.len(),
        );
    } else {
        println!("locals: {} 件すべて設定済み。", decls.len());
    }
    Ok(())
}
