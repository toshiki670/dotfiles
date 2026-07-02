//! `dotfiles list`：configs の分散 manifest を集約し、配置先一覧を表示する。
//!
//! 設計書 §11 の「分散方式の弱点（全体一覧が横断的）を補う」俯瞰ビュー。各単位を
//! source 相対名でソートし、`名前 → dst [属性]` の形で出力する。配置は行わないため
//! `home` は不要（dst は manifest の生表記 `~/...` をそのまま見せる方が読みやすい）。
//!
//! 見出しには `source` の実 path でなく解決元ラベル（`origin`・[`crate::core::source::Origin`]）を出す。
//! 埋め込み時の `source` は temp dir で、実 path を生で見せても俯瞰の役に立たないため（§8）。

use crate::core::discover::{self, MANIFEST};
use crate::core::manifest::Manifest;
use std::path::Path;

/// `source` 配下の設定単位を一覧表示する。`origin` は見出しに出す解決元ラベル（§8）。
pub fn run(source: &Path, origin: &str) -> Result<(), String> {
    let units = discover::collect(source)?;
    if units.is_empty() {
        println!(
            "list: 対象なし（{} に manifest.toml がない）",
            source.display()
        );
        return Ok(());
    }

    let mut rows = Vec::with_capacity(units.len());
    for unit in &units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        rows.push((unit.rel.to_string_lossy().into_owned(), manifest));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    let width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    println!("dotfiles list（source: {origin}）");
    for (name, manifest) in &rows {
        println!(
            "  {name:<width$}  → {dst}  [{attrs}]",
            dst = manifest.dst,
            attrs = attrs(manifest),
        );
    }
    Ok(())
}

/// 1 単位の属性ラベル（2軸モデル, §7）。
/// kind ＋ strategy ＋ overlay 数 ＋ preserve ＋ private / executable ＋ when.deps / when.os / when.profile ＋ hooks。
fn attrs(manifest: &Manifest) -> String {
    // 表示名は Kind / Strategy の Display に集約する（apply のラベルと同じ出所）。
    let mut parts = vec![manifest.kind.to_string()];
    if let Some(strategy) = manifest.strategy {
        parts.push(strategy.to_string());
    }
    if !manifest.overlay.is_empty() {
        parts.push(format!("overlay={}", manifest.overlay.len()));
    }
    if manifest.preserve {
        parts.push("preserve".to_string());
    }
    if manifest.private {
        parts.push("private".to_string());
    }
    if manifest.executable {
        parts.push("executable".to_string());
    }
    if let Some(when) = &manifest.when {
        if !when.deps.is_empty() {
            parts.push(format!("when.deps={}", when.deps.join("+")));
        }
        if let Some(os) = &when.os {
            parts.push(format!("when.os={os}"));
        }
        if let Some(profile) = &when.profile {
            parts.push(format!("when.profile={profile}"));
        }
    }
    if !manifest.hooks.is_empty() {
        // フックはコマンド（argv）なので、一覧では件数だけ示す（詳細は manifest を見る）。
        // always 頻度（#546）が混じる場合だけ内訳を付す（onchange のみなら黙示の既定のまま）。
        let always = manifest
            .hooks
            .iter()
            .filter(|h| h.frequency == crate::core::manifest::Frequency::Always)
            .count();
        if always > 0 {
            parts.push(format!("hooks={} (always={always})", manifest.hooks.len()));
        } else {
            parts.push(format!("hooks={}", manifest.hooks.len()));
        }
    }
    parts.join(", ")
}
