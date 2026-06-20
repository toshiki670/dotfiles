//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する。
//!
//! 走査（manifest 発見・再帰委譲）は [`crate::discover`]、実体配置は kind ごとに
//! [`crate::copy`]（copy 層）/ [`crate::generate`]（generate 層）/ [`crate::merge`]（merge 層）へ
//! 委譲する。本モジュールはオーケストレーション（単位ごとに kind で分岐し結果を表示）と、
//! 各層が共有する小道具（dst の `~` 展開・パーミッション適用）だけを持つ。hooks は後続スライス。

use crate::discover::{self, MANIFEST, Unit};
use crate::manifest::{Kind, Manifest};
use crate::{copy, generate, merge};
use std::path::{Path, PathBuf};

/// 1 単位の配置結果。配置済みと「gate でスキップ」を表示で区別するために返す。
pub enum Outcome {
    /// 配置/生成した。
    Placed,
    /// 条件（deps gate / os 等）を満たさず配置しなかった。理由を持つ。
    Skipped(String),
}

/// `source`（= `configs/`）配下を走査し、各 manifest の配置を実行する。
/// `home` は dst の `~` 展開先。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    if units.is_empty() {
        println!(
            "apply: 対象なし（{} に manifest.toml がない）",
            source.display()
        );
        return Ok(());
    }

    for unit in &units {
        apply_unit(unit, home)?;
    }
    Ok(())
}

/// 1 単位を kind で分岐して配置し、結果を 1 行で表示する。
fn apply_unit(unit: &Unit, home: &Path) -> Result<(), String> {
    let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
    let dst = expand_home(&manifest.dst, home);
    let (kind, outcome) = match manifest.kind {
        Kind::Copy => ("copy", copy::place(&unit.dir, &dst, &manifest)?),
        Kind::Generate => ("generate", generate::place(&unit.dir, &dst, &manifest)?),
        Kind::Merge => ("merge", merge::place(&unit.dir, &dst, &manifest)?),
    };

    let name = unit.rel.to_string_lossy();
    match outcome {
        Outcome::Placed => println!("apply: {name} → {} ({kind})", manifest.dst),
        Outcome::Skipped(why) => println!("apply: {name} → skip ({kind}: {why})"),
    }
    Ok(())
}

/// 配置済みファイルへ manifest のパーミッションを適用する（Unix のみ）。
/// copy / generate 両層が共有する。
#[cfg(unix)]
pub fn set_mode(dst: &Path, manifest: &Manifest) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dst, std::fs::Permissions::from_mode(manifest.mode()))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", dst.display()))
}

/// 非 Unix では no-op（パーミッションモデルが異なるため）。
#[cfg(not(unix))]
pub fn set_mode(_dst: &Path, _manifest: &Manifest) -> Result<(), String> {
    Ok(())
}

/// dst の `~` / `~/...` を `home` に展開する。
/// `$XDG_*` 等の正規化は設計書 §14 で確定。
fn expand_home(dst: &str, home: &Path) -> PathBuf {
    if let Some(rest) = dst.strip_prefix("~/") {
        home.join(rest)
    } else if dst == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(dst)
    }
}
