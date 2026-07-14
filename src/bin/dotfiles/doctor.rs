//! `dotfiles doctor`: 全ユニットの `locals` 宣言・ユニット間の配置先衝突・不要になった配置を診断する。
//!
//! 全単位の `manifest.toml`（[`crate::discover`]）から `locals` を集め、ストア
//! （[`crate::locals::store`]）に値が無い名前を報告する。加えて [`crate::placements::expected`] が
//! 導出する期待配置集合から、複数ユニットが同一パスへ output を宣言している箇所（#593）を報告する。
//! さらに [`crate::prune::stale`] が示す「前回 apply 時点は期待されていたが今回はもう期待されない」
//! 配置（#521）も報告する ― 実削除はしない（`dotfiles apply --force` の役目）。いずれもツール別
//! ロジックは持たず、宣言と現状の突合だけを見る（診断の拡張は #576）。問題があっても
//! **ブロックしない**（exit 0・情報提供）。

use crate::apply::gate;
use crate::discover::{self, MANIFEST};
use crate::locals::store::Store;
use crate::manifest::{Manifest, OutputSource, Step, Steps};
use crate::placements::{self, Placement};
use crate::prune;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `source` 配下の `locals` 宣言・期待配置集合を突き合わせ、未設定・衝突・不要な配置を stderr へ
/// 報告する。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    let store = Store::load(home)?;
    report_missing_locals(&units, &store)?;

    let expected = placements::expected(source, home)?;
    report_placement_conflicts(&expected);

    let gate_state = gate::GateState::load(home)?;
    let current =
        placements::expected_gated(source, home, &|w| gate::when_satisfied(w, &gate_state))?;
    let stale = prune::stale(home, &current);
    report_stale_placements(source, home, &gate_state, &stale);

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
/// 報告する（#593）。ファイル粒度の比較なので conf.d 等の合流点は誤検知しない（理由は
/// [`placements::expected`] のモジュール doc）。
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
    // ソートして出力を決定的にする。
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

/// `stale`（[`prune::stale`]）を報告する。実削除はしない（`dotfiles apply --force` の役目）。
/// 各パスに [`stale_reason`] で理由（gate 不成立の内容、または configs からの削除）を添える。
fn report_stale_placements(
    source: &Path,
    home: &Path,
    gate_state: &gate::GateState,
    stale: &[PathBuf],
) {
    if stale.is_empty() {
        println!("doctor: 不要になった配置はありません");
        return;
    }
    eprintln!("doctor: 不要になった配置が {} 件あります:", stale.len());
    for path in stale {
        let reason = stale_reason(source, home, gate_state, path);
        eprintln!("  - {}（{reason}）", path.display());
    }
    eprintln!("  `dotfiles apply --force` で退避できます。");
}

/// `path` が stale になった理由の人間向け説明。表示の親切さのためだけの診断で、除去条件の判定
/// （[`prune::stale`]）そのものには影響しない。
///
/// 今も宣言されている（gate 評価なしの [`placements::expected`]）ユニットが見つかれば、その
/// ユニットの gate（unit → output step の順）を再評価して理由を言う。見つからなければ、ユニット
/// 削除かツリーファイル削除で configs 側から消えたとみなす。
fn stale_reason(source: &Path, home: &Path, gate_state: &gate::GateState, path: &Path) -> String {
    let declared = placements::expected(source, home).unwrap_or_default();
    let Some(owner) = declared
        .iter()
        .find(|p| p.path == path)
        .map(|p| p.unit.clone())
    else {
        return "configs から削除された可能性があります".to_string();
    };

    let manifest = match Manifest::load(&source.join(&owner).join(MANIFEST)) {
        Ok(m) => m,
        Err(_) => return format!("{owner}: manifest の再読み込みに失敗しました"),
    };
    if let Some(reason) = gate::unit_skip_reason(&manifest, gate_state) {
        return format!("{owner}: {reason}");
    }
    let step_gated = match &manifest.steps {
        // ツリー配置は step を持たず when も書けない（unit gate のみ）。
        Steps::Tree { .. } => false,
        Steps::Pipeline { steps, .. } => steps.iter().any(|step| match step {
            Step::Output(out) => {
                matches!(&out.dest, OutputSource::Path(p) if p.resolve(home) == path)
                    && !gate::when_satisfied(&out.when, gate_state)
            }
            Step::Input(_) => false,
        }),
    };
    if step_gated {
        return format!("{owner}: output step の when が不成立です");
    }
    format!("{owner}: 原因不明")
}
