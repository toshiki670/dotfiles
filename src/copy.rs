//! `dotfiles apply` の copy 層: ディレクトリ単位で実体をそのまま書き出す。
//!
//! 設計書 §5 の3層のうち大多数を占める層。`dst`（ディレクトリ）の下へ、単位ディレクトリ
//! 配下の実ファイルを相対構造を保ったまま copy する。`manifest.toml` 自体と、別 manifest を
//! 持つサブツリー（委譲先の責務）は除外する。パーミッションは manifest の
//! `private` / `executable` 属性に従う（§7、適用は [`crate::apply::set_mode`]）。

use crate::apply::set_mode;
use crate::discover::{self, MANIFEST};
use crate::manifest::Manifest;
use std::path::Path;

/// 1 単位（`dir`）を copy で `dst`（ディレクトリ）へ配置する。
pub fn place(dir: &Path, dst: &Path, manifest: &Manifest) -> Result<(), String> {
    copy_tree(dir, dir, dst, manifest)
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
