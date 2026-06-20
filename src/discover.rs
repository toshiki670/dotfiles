//! configs ソースツリーから設定単位（`manifest.toml` を持つディレクトリ）を発見する。
//!
//! `apply`（配置）と `list`（一覧）が共有する走査ロジック。設計書 §6.3 ルール1 の
//! 「管轄の再帰委譲」をここで実装する: あるディレクトリに `manifest.toml` があれば
//! それを 1 単位とし、サブディレクトリに別の `manifest.toml` があればそこも別単位として
//! 収集する（ツリーを manifest で分割統治する）。

use std::path::{Path, PathBuf};

/// 設定単位を示すファイル名。これを持つディレクトリが 1 単位になる。
pub const MANIFEST: &str = "manifest.toml";

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）。
pub struct Unit {
    /// 単位のディレクトリ実パス（`source` を基点とする）。
    pub dir: PathBuf,
    /// `source` ルートからの相対パス。表示名（`dotfiles list`）に使う。
    pub rel: PathBuf,
}

/// `source`（= `configs/`）配下を走査し、設定単位を収集する。
///
/// `source` が存在しない場合は、移動先を案内するエラーを返す（apply / list 共通）。
pub fn collect(source: &Path) -> Result<Vec<Unit>, String> {
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
    walk(source, source, &mut units)?;
    Ok(units)
}

/// `dir` 以下を再帰的にたどり、`manifest.toml` を持つディレクトリを単位として収集する。
fn walk(dir: &Path, source: &Path, out: &mut Vec<Unit>) -> Result<(), String> {
    if dir.join(MANIFEST).is_file() {
        let rel = dir.strip_prefix(source).unwrap_or(dir).to_path_buf();
        out.push(Unit {
            dir: dir.to_path_buf(),
            rel,
        });
    }
    for entry in read_dir(dir)? {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, source, out)?;
        }
    }
    Ok(())
}

/// `std::fs::read_dir` を `Vec<DirEntry>` に集約しつつエラーメッセージを整える。
pub fn read_dir(dir: &Path) -> Result<Vec<std::fs::DirEntry>, String> {
    let mut entries = Vec::new();
    let iter =
        std::fs::read_dir(dir).map_err(|e| format!("{}: 読み込み失敗: {e}", dir.display()))?;
    for entry in iter {
        entries.push(entry.map_err(|e| format!("{}: エントリ取得失敗: {e}", dir.display()))?);
    }
    Ok(entries)
}
