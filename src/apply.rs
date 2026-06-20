//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する。
//!
//! 走査（manifest 発見・再帰委譲）は [`crate::discover`]。各単位は §5.5 の評価順に従う:
//! ①**ユニット gate（`deps` / `os`）を最初に評価し false なら短絡**（[`crate::gate`]、dst を
//! 一切触らず skip）。生き残った単位は dst 種別で配置経路が分かれる ―
//! dst=ディレクトリの copy は [`crate::copy`]（ツリー配置）、dst=ファイルの generate /
//! overlay 明示は [`crate::compose`]（②宣言順 overlay ③preserve 最後の合成）。本モジュールは
//! オーケストレーションと、両経路が共有する小道具（`~` 展開・パーミッション適用）を持つ。

use crate::discover::{self, MANIFEST, Unit};
use crate::manifest::{Kind, Manifest, Strategy};
use crate::{compose, copy, gate};
use std::path::{Path, PathBuf};

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

/// 1 単位を評価順（§5.5）に従って配置し、結果を 1 行で表示する。
fn apply_unit(unit: &Unit, home: &Path) -> Result<(), String> {
    let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
    let dst = expand_home(&manifest.dst, home);
    let name = unit.rel.to_string_lossy();

    // ①ユニット gate を最初に評価し、満たさなければユニット全体を skip（dst は触らない）。
    if let Some(reason) = gate::unit_skip_reason(&manifest) {
        println!("apply: {name} → skip ({reason})");
        return Ok(());
    }

    let label = placement_label(&manifest);
    if uses_compose(&manifest) {
        compose::place(&unit.dir, &dst, &manifest)?;
    } else {
        copy::place(&unit.dir, &dst, &manifest)?;
    }
    println!("apply: {name} → {} ({label})", manifest.dst);
    Ok(())
}

/// ファイル合成経路（[`crate::compose`]）を通すか。overlay 明示、または dst=ファイルの
/// generate はファイル合成。それ以外（overlay 無しの copy）は copy ツリー配置。
fn uses_compose(manifest: &Manifest) -> bool {
    !manifest.overlay.is_empty() || manifest.kind == Kind::Generate
}

/// 表示用の配置ラベル（apply の 1 行出力）。overlay 明示は strategy を併記する。
fn placement_label(manifest: &Manifest) -> &'static str {
    if !manifest.overlay.is_empty() {
        return match manifest.strategy {
            Some(Strategy::JsonShallow) => "overlay/json-shallow",
            Some(Strategy::Concat) => "overlay/concat",
            None => "overlay",
        };
    }
    match manifest.kind {
        Kind::Copy => "copy",
        Kind::Generate => "generate",
    }
}

/// 配置済みファイルへ manifest のパーミッションを適用する（Unix のみ）。
/// copy / compose 両経路が共有する。
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
