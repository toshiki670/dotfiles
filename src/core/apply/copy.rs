//! `dotfiles apply` の copy 層: ディレクトリ単位で実体をそのまま書き出す。
//!
//! copy はユニットのソースツリーを相対構造を保ったまま `dst`（ディレクトリ）の下へ書き出す
//! 生成方式で、overlay を持たないユニットの大多数がこの経路を通る。`manifest.toml` 自体と、
//! 別 manifest を持つサブツリー（委譲先の責務）は除外する。パーミッションは manifest の
//! `private` / `executable` 属性に従う（適用は [`crate::core::apply::set_mode`]）。
//! `locals`（named value）が解決されている単位では、各ファイルへ `@@name@@` 注入を通す。

use super::set_mode;
use crate::core::discover::{self, MANIFEST};
use crate::core::locals::resolve;
use crate::core::manifest::Manifest;
use std::collections::BTreeMap;
use std::path::Path;

/// 1 単位（`dir`）を copy で `dst`（ディレクトリ）へ配置する。
/// `locals` は解決済みの named value（空なら注入なし＝従来の純コピー）。
pub fn place(
    dir: &Path,
    dst: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
) -> Result<(), String> {
    copy_tree(dir, dir, dst, manifest, locals)
}

/// `src_root` 配下の実ファイルを、相対構造を保ったまま `dst_root` へコピーする。
/// `manifest.toml` 自体と、別 manifest を持つサブツリーは除外する（委譲先の責務）。
fn copy_tree(
    current: &Path,
    src_root: &Path,
    dst_root: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
) -> Result<(), String> {
    for entry in discover::read_dir(current)? {
        let path = entry.path();
        if path.is_dir() {
            // サブ manifest を持つディレクトリは別単位なので委譲（コピー対象外）。
            if path.join(MANIFEST).is_file() {
                continue;
            }
            copy_tree(&path, src_root, dst_root, manifest, locals)?;
        } else if entry.file_name() != MANIFEST {
            let rel = path
                .strip_prefix(src_root)
                .map_err(|e| format!("{}: 相対パス算出失敗: {e}", path.display()))?;
            copy_file(&path, &dst_root.join(rel), manifest, locals)?;
        }
    }
    Ok(())
}

/// 親ディレクトリを作りつつ 1 ファイルを配置し、manifest のパーミッションを与える。
/// `locals` が空なら高速な `fs::copy`、非空なら read→注入→write で `@@name@@` を埋める。
fn copy_file(
    src: &Path,
    dst: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    if locals.is_empty() {
        std::fs::copy(src, dst)
            .map_err(|e| format!("{} → {}: コピー失敗: {e}", src.display(), dst.display()))?;
    } else {
        let bytes =
            std::fs::read(src).map_err(|e| format!("{}: 読み込み失敗: {e}", src.display()))?;
        let injected = resolve::inject(&bytes, locals);
        std::fs::write(dst, &injected)
            .map_err(|e| format!("{}: 書き込み失敗: {e}", dst.display()))?;
    }
    set_mode(dst, manifest)?;
    Ok(())
}
