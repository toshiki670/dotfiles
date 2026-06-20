//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する（S0 最小骨格）。
//!
//! S0 はソース解決（検出 / 判定 / 埋め込み）を持たない（→ S8）。呼び出し側が渡す
//! 固定パスをそのまま読む。走査は `manifest.toml` を持つディレクトリ単位で、copy
//! のみを実行する。複数ツール対応や `dotfiles list` は S1 以降。

use crate::manifest::{Kind, Manifest};
use std::path::{Path, PathBuf};

const MANIFEST: &str = "manifest.toml";

/// `source`（= `configs/`）配下を走査し、各 manifest の配置を実行する。
/// `home` は dst の `~` 展開先。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    if !source.is_dir() {
        let looked = std::env::current_dir()
            .map(|cwd| cwd.join(source))
            .unwrap_or_else(|_| source.to_path_buf());
        return Err(format!(
            "ソース {src} が見つかりません（探索先: {looked}）。\n\
             {src} はカレントディレクトリからの相対パスです。\
             configs/ のあるリポジトリのルートに移動してから実行してください。",
            src = source.display(),
            looked = looked.display(),
        ));
    }

    let mut units = Vec::new();
    collect_manifests(source, &mut units)?;
    if units.is_empty() {
        println!(
            "apply: 対象なし（{} に manifest.toml がない）",
            source.display()
        );
        return Ok(());
    }

    for dir in units {
        apply_unit(&dir, home)?;
    }
    Ok(())
}

/// `manifest.toml` を持つディレクトリ（= 設定単位）を再帰的に収集する。
/// サブディレクトリに別の manifest があればそれも別ユニットとして収集する
/// （ツリーを manifest で分割統治。設計書 §6.3 ルール1）。
fn collect_manifests(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if dir.join(MANIFEST).is_file() {
        out.push(dir.to_path_buf());
    }
    for entry in read_dir(dir)? {
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, out)?;
        }
    }
    Ok(())
}

/// 1 ユニットを配置する。S0 は kind=copy のみ。
fn apply_unit(dir: &Path, home: &Path) -> Result<(), String> {
    let manifest = Manifest::load(&dir.join(MANIFEST))?;
    let dst = expand_home(&manifest.dst, home);
    match manifest.kind {
        Kind::Copy => copy_tree(dir, dir, &dst)?,
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
fn copy_tree(current: &Path, src_root: &Path, dst_root: &Path) -> Result<(), String> {
    for entry in read_dir(current)? {
        let path = entry.path();
        if path.is_dir() {
            // サブ manifest を持つディレクトリは別ユニットなので委譲（コピー対象外）。
            if path.join(MANIFEST).is_file() {
                continue;
            }
            copy_tree(&path, src_root, dst_root)?;
        } else if entry.file_name() != MANIFEST {
            let rel = path
                .strip_prefix(src_root)
                .map_err(|e| format!("{}: 相対パス算出失敗: {e}", path.display()))?;
            copy_file(&path, &dst_root.join(rel))?;
        }
    }
    Ok(())
}

/// 親ディレクトリを作りつつ 1 ファイルをコピーする。
fn copy_file(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::copy(src, dst)
        .map_err(|e| format!("{} → {}: コピー失敗: {e}", src.display(), dst.display()))?;
    Ok(())
}

/// dst の `~` / `~/...` を `home` に展開する。
/// S0 は `~` のみ対応（`$XDG_*` 等の正規化は設計書 §14 で確定）。
fn expand_home(dst: &str, home: &Path) -> PathBuf {
    if let Some(rest) = dst.strip_prefix("~/") {
        home.join(rest)
    } else if dst == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(dst)
    }
}

/// `std::fs::read_dir` を `Vec<DirEntry>` に集約しつつエラーメッセージを整える。
fn read_dir(dir: &Path) -> Result<Vec<std::fs::DirEntry>, String> {
    let mut entries = Vec::new();
    let iter =
        std::fs::read_dir(dir).map_err(|e| format!("{}: 読み込み失敗: {e}", dir.display()))?;
    for entry in iter {
        entries.push(entry.map_err(|e| format!("{}: エントリ取得失敗: {e}", dir.display()))?);
    }
    Ok(entries)
}
