//! `dotfiles doctor`: 全ユニットの `locals` 宣言・ユニット間の配置先衝突・現在 profile の宣言状況・
//! 不要になった配置を診断する。
//!
//! 全単位の `manifest.toml`（[`crate::discover`]）から `locals` を集め、ストア
//! （[`crate::locals::store`]）に値が無い名前を報告する。加えて [`crate::placements::expected`] が
//! 導出する期待配置集合から、複数ユニットが同一パスへ output を宣言している箇所（#593）を報告する。
//! 現在の profile 状態が、全 unit の `when.profile` 宣言値集合に現れているかも突合する（#628・
//! typo 検出）。さらに [`crate::prune::stale`] が示す「前回 apply 時点は期待されていたが今回はもう
//! 期待されない」配置（#521）も報告する ― 実削除はしない（`dotfiles apply --force` の役目）。
//! いずれもツール別ロジックは持たず、宣言と現状の突合だけを見る（診断の拡張は #576）。問題があっても
//! **ブロックしない**（exit 0・情報提供）。

use crate::apply::gate;
use crate::discover::{self, MANIFEST};
use crate::locals::store::Store;
use crate::manifest::{Manifest, OutputSource, Step, Steps, When};
use crate::placements::{self, Placement};
use crate::prune;
use std::collections::{BTreeMap, BTreeSet};
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
    report_unknown_profile(&units, gate_state.profile())?;
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

/// 現在の profile 状態が、全 unit の `when.profile` 宣言値集合に現れない場合に警告する（#628）。
/// 未設定（`current` が None）は正常系（not-private 既定）のため対象外 ― 完全に沈黙する。
///
/// 「どの manifest も参照しない profile 値」は正当な使い方でもあり得る（private を避ける目的だけの
/// 値等）ため、エラーにはせず typo の可能性を知らせる情報提供に留める。
fn report_unknown_profile(units: &[discover::Unit], current: Option<&str>) -> Result<(), String> {
    let Some(current) = current else {
        return Ok(());
    };

    let mut declared = BTreeSet::new();
    for unit in units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        collect_profiles(&manifest, &mut declared);
    }

    if declared.contains(current) {
        println!("doctor: profile `{current}` を参照する when.profile があります");
        return Ok(());
    }
    eprintln!(
        "doctor: profile `{current}` を参照する when.profile がありません（typo であれば意図した \
         profile gate 付きの断片が配置されません。意図した値であれば無視してください）"
    );
    Ok(())
}

/// `manifest` のトップレベル `when` と（パイプラインなら）各 step の `when` から `profile` 宣言値を
/// `into` へ集める。ツリー配置（[`Steps::Tree`]）は step の `when` を持てないためトップレベルのみ。
fn collect_profiles(manifest: &Manifest, into: &mut BTreeSet<String>) {
    collect_when_profile(&manifest.when, into);
    if let Steps::Pipeline { steps, .. } = &manifest.steps {
        for step in steps {
            let when = match step {
                Step::Input(s) => &s.when,
                Step::Output(s) => &s.when,
            };
            collect_when_profile(when, into);
        }
    }
}

fn collect_when_profile(when: &Option<When>, into: &mut BTreeSet<String>) {
    if let Some(profile) = when.as_ref().and_then(|w| w.profile.as_deref()) {
        into.insert(profile.to_string());
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
