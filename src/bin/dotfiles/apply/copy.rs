//! `dotfiles apply` の copy 層: ディレクトリ単位で実体をそのまま書き出す。
//!
//! ツリー output（`input = "."` ＋ パス output）の配置経路。ユニットのソースツリーを相対構造を
//! 保ったまま宛先ディレクトリの下へ書き出す。`manifest.toml` 自体と、別 manifest を持つサブツリー
//! （委譲先の責務）は除外する。書き込みは [`crate::apply::write_if_changed`] を通し、byte-identical な
//! ファイルは再 apply でも mtime を保つ（パス output と同じ冪等最適化）。パーミッションは manifest の
//! `private` / `executable` 属性に従う（適用は [`crate::apply::set_mode`]）。`locals`（named value）が
//! 解決されている単位では、各ファイルへ `@@name@@` 注入を通す。[`crate::apply::pipeline`] が内容が
//! ツリーのときに呼ぶ。

use super::{set_mode, write_if_changed};
use crate::discover::{self, MANIFEST};
use crate::locals::resolve;
use crate::manifest::Manifest;
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

/// 1 ファイルを配置し、manifest のパーミッションを与える。`src` を読み（`locals` が非空なら
/// `@@name@@` を注入して）最終バイトを作り、[`crate::apply::write_if_changed`] で親ディレクトリを
/// 作りつつ冪等に書く（byte-identical なら書き込みを省いて mtime を保つ）。書き込みを省いても
/// mode は毎回再適用する（属性変更が反映されるように・パス output と対称）。
fn copy_file(
    src: &Path,
    dst: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| format!("{}: 読み込み失敗: {e}", src.display()))?;
    let final_bytes = if locals.is_empty() {
        bytes
    } else {
        resolve::inject(&bytes, locals)
    };
    write_if_changed(dst, &final_bytes)?;
    set_mode(dst, manifest)?;
    Ok(())
}
