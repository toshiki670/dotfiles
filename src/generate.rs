//! generate 層（§5 generate）: コマンドを実行し、その標準出力を断片にする。
//!
//! 補完5本（gh / docker / bat / clip / merge-ready）が対象。dst=ファイルで、`cmd` の標準出力に
//! 同ディレクトリの sibling（`manifest.toml` 以外）を連結する。連結は合成軸の `concat` 戦略へ
//! 統一した（[`crate::strategy::concat`]、出力は従来と不変）。`when.deps` gate は apply 前段の
//! ユニット gate（[`crate::gate`]）へ移し、実体の合成・書き込みは [`crate::compose`] が担う。
//! 本モジュールはコマンド実行（`run_cmd`）と、既定断片列（cmd 出力＋sibling）の組み立てだけを持つ。

use crate::discover::{self, MANIFEST};
use crate::manifest::Manifest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// generate（overlay 無し）の既定断片列を作る: `cmd` 出力を先頭に、sibling を名前順で続ける。
/// `compose` がこれを `concat` 戦略で 1 ファイルへ束ねる。`cmd` 未指定はエラー。
pub fn default_fragments(dir: &Path, manifest: &Manifest) -> Result<Vec<Vec<u8>>, String> {
    if manifest.cmd.is_empty() {
        return Err(format!(
            "{}: generate には cmd が必要です",
            dir.join(MANIFEST).display()
        ));
    }
    let mut frags = vec![run_cmd(&manifest.cmd)?];
    frags.extend(sibling_fragments(dir)?);
    Ok(frags)
}

/// `cmd`（argv）を実行し標準出力を返す。非ゼロ終了は stderr 付きでエラーにする。
/// overlay の generate 断片（`cmd`）からも使う。
pub fn run_cmd(cmd: &[String]) -> Result<Vec<u8>, String> {
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| format!("{}: 実行失敗: {e}", cmd[0]))?;
    if !output.status.success() {
        return Err(format!(
            "{cmd:?} が異常終了 ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// 単位ディレクトリ直下の `manifest.toml` 以外のファイルを名前順に読み、断片列にする。
/// gh の独自補完ブロック（`custom.fish`）など、生成物の後ろへ接ぐ静的断片に使う。
fn sibling_fragments(dir: &Path) -> Result<Vec<Vec<u8>>, String> {
    let mut files: Vec<PathBuf> = discover::read_dir(dir)?
        .into_iter()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.file_name() != Some(OsStr::new(MANIFEST)))
        .collect();
    files.sort();

    let mut frags = Vec::with_capacity(files.len());
    for f in &files {
        frags.push(std::fs::read(f).map_err(|e| format!("{}: 読み込み失敗: {e}", f.display()))?);
    }
    Ok(frags)
}
