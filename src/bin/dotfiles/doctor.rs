//! `dotfiles doctor`: 全ユニットの `locals` 宣言と、ユニット間の配置先衝突を診断する。
//!
//! 全単位の `manifest.toml`（[`crate::discover`]）から `locals` を集め、ストア
//! （[`crate::locals::store`]）に値が無い名前を報告する。加えて [`crate::placements::expected`] が
//! 導出する期待配置集合から、複数ユニットが同一パスへ output を宣言している箇所（#593）を報告する。
//! いずれもツール別ロジックは持たず、宣言と現状の突合だけを見る（診断の拡張は #576）。問題があっても
//! **ブロックしない**（exit 0・情報提供）。

use crate::discover::{self, MANIFEST};
use crate::locals::store::Store;
use crate::manifest::Manifest;
use crate::placements::{self, Placement};
use std::collections::BTreeMap;
use std::path::Path;

/// `source` 配下の `locals` 宣言・期待配置集合を突き合わせ、未設定・衝突を stderr に報告する。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    let store = Store::load(home)?;
    report_missing_locals(&units, &store)?;

    let expected = placements::expected(source, home)?;
    report_placement_conflicts(&expected);

    Ok(())
}

/// 全単位の `locals` 宣言のうち、ストアに値が無い名前を報告する。
fn report_missing_locals(units: &[discover::Unit], store: &Store) -> Result<(), String> {
    let mut missing = Vec::new();
    for unit in units {
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

/// 期待配置集合を path でグルーピングし、2 ユニット以上が同一パスへ output を宣言している箇所を
/// 報告する（#593）。conf.d 等の合流点はファイル粒度で比較するため、別ファイル名を置き合う正当な
/// 合流は対象に入らない（[`placements::expected`] が既にツリーをファイル展開済み）。
fn report_placement_conflicts(expected: &[Placement]) {
    let mut by_path: BTreeMap<&Path, Vec<&str>> = BTreeMap::new();
    for p in expected {
        let units = by_path.entry(p.path.as_path()).or_default();
        if !units.contains(&p.unit.as_str()) {
            units.push(&p.unit);
        }
    }
    let mut conflicts: Vec<_> = by_path.into_iter().filter(|(_, u)| u.len() >= 2).collect();
    // ユニット名は discover::collect の走査順（read_dir 由来・未ソート）で積まれるため、表示前に
    // ソートして出力を決定的にする（走査順が環境で揺れても衝突行の並びは揺れない）。
    for (_, units) in &mut conflicts {
        units.sort_unstable();
    }

    if conflicts.is_empty() {
        println!("doctor: 配置先の衝突はありません");
        return;
    }
    eprintln!("doctor: 配置先の衝突が {} 件あります:", conflicts.len());
    for (path, units) in &conflicts {
        eprintln!("  - {}: {}", path.display(), units.join(", "));
    }
}
