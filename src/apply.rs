//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する。
//!
//! 走査（manifest 発見・再帰委譲）は [`crate::discover`] に委譲する。本モジュールは
//! copy 層の実体配置を担う: ディレクトリ単位 copy・複数ファイル・サブディレクトリ再帰、
//! および manifest の `private` / `executable` 属性に基づくパーミッション付与（§7）。
//! ソース解決（検出 / 判定 / 埋め込み）は持たない（→ S8）。generate / merge / hooks も
//! 後続スライス。

use crate::discover::{self, MANIFEST};
use crate::manifest::{Kind, Manifest};
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
        apply_unit(&unit.dir, home)?;
    }
    Ok(())
}

/// 1 単位を配置する。S1 は kind=copy のみ。
fn apply_unit(dir: &Path, home: &Path) -> Result<(), String> {
    let manifest = Manifest::load(&dir.join(MANIFEST))?;
    let dst = expand_home(&manifest.dst, home);
    match manifest.kind {
        Kind::Copy => copy_tree(dir, dir, &dst, &manifest)?,
    }
    println!(
        "apply: {} → {} (copy)",
        dir.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
        manifest.dst,
    );
    Ok(())
}

/// `src_root` 配下の実ファイルを、相対構造を保ったまま `dst_root` へコピーする。
/// `manifest.toml` 自体と、別 manifest を持つサブツリーは除外する（委譲先の責務）。
fn copy_tree(
    current: &Path,
    src_root: &Path,
    dst_root: &Path,
    manifest: &Manifest,
) -> Result<(), String> {
    for entry in discover::read_dir(current)? {
        let path = entry.path();
        if path.is_dir() {
            // サブ manifest を持つディレクトリは別単位なので委譲（コピー対象外）。
            if path.join(MANIFEST).is_file() {
                continue;
            }
            copy_tree(&path, src_root, dst_root, manifest)?;
        } else if entry.file_name() != MANIFEST {
            let rel = path
                .strip_prefix(src_root)
                .map_err(|e| format!("{}: 相対パス算出失敗: {e}", path.display()))?;
            copy_file(&path, &dst_root.join(rel), manifest)?;
        }
    }
    Ok(())
}

/// 親ディレクトリを作りつつ 1 ファイルをコピーし、manifest のパーミッションを与える。
fn copy_file(src: &Path, dst: &Path, manifest: &Manifest) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::copy(src, dst)
        .map_err(|e| format!("{} → {}: コピー失敗: {e}", src.display(), dst.display()))?;
    set_mode(dst, manifest)?;
    Ok(())
}

/// 配置済みファイルへ manifest のパーミッションを適用する（Unix のみ）。
#[cfg(unix)]
fn set_mode(dst: &Path, manifest: &Manifest) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dst, std::fs::Permissions::from_mode(manifest.mode()))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", dst.display()))
}

/// 非 Unix では no-op（パーミッションモデルが異なるため）。
#[cfg(not(unix))]
fn set_mode(_dst: &Path, _manifest: &Manifest) -> Result<(), String> {
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
