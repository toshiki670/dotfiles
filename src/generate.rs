//! `dotfiles apply` の generate 層: コマンドを実行して 1 ファイルを生成・配置する。
//!
//! 設計書 §5（generate）/ §7（deps）。補完5本（gh / docker / bat / clip / merge-ready）が
//! 対象。copy 層（dst=ディレクトリ・実体コピー）と異なり、generate は **dst=ファイル**で、
//! `cmd` の標準出力をそこへ書き出す。単位ディレクトリに `manifest.toml` 以外のファイルが
//! あれば、生成物の後ろへ連結する（gh の独自補完ブロックの注入に使う）。
//! `deps` の依存バイナリが PATH に揃わないときは生成をスキップする（gate）。

use crate::apply::{Outcome, set_mode};
use crate::discover::{self, MANIFEST};
use crate::manifest::Manifest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 1 単位（`dir`）を generate で `dst`（ファイル）へ生成・配置する。
pub fn place(dir: &Path, dst: &Path, manifest: &Manifest) -> Result<Outcome, String> {
    if manifest.cmd.is_empty() {
        return Err(format!(
            "{}: generate には cmd が必要です",
            dir.join(MANIFEST).display()
        ));
    }
    // deps gate（§7）: PATH に無い依存が 1 つでもあれば生成しない。
    if let Some(missing) = first_missing_dep(&manifest.deps) {
        return Ok(Outcome::Skipped(format!("依存 `{missing}` が PATH にない")));
    }

    let mut bytes = run_cmd(&manifest.cmd)?;
    append_siblings(dir, &mut bytes)?;

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(dst, &bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", dst.display()))?;
    set_mode(dst, manifest)?;
    Ok(Outcome::Placed)
}

/// `cmd`（argv）を実行し標準出力を返す。非ゼロ終了は stderr 付きでエラーにする。
fn run_cmd(cmd: &[String]) -> Result<Vec<u8>, String> {
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

/// 単位ディレクトリ直下の `manifest.toml` 以外のファイルを、名前順に `bytes` へ連結する。
/// 連結の境目には改行を 1 つ補い、生成物と静的ブロックが行頭で接ぐようにする。
fn append_siblings(dir: &Path, bytes: &mut Vec<u8>) -> Result<(), String> {
    let mut files: Vec<PathBuf> = discover::read_dir(dir)?
        .into_iter()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.file_name() != Some(OsStr::new(MANIFEST)))
        .collect();
    files.sort();

    for f in &files {
        if bytes.last().is_some_and(|&b| b != b'\n') {
            bytes.push(b'\n');
        }
        let content =
            std::fs::read(f).map_err(|e| format!("{}: 読み込み失敗: {e}", f.display()))?;
        bytes.extend_from_slice(&content);
    }
    Ok(())
}

/// `deps` のうち最初に PATH 上で見つからないものを返す（全て揃えば None）。
fn first_missing_dep(deps: &[String]) -> Option<&str> {
    deps.iter()
        .map(String::as_str)
        .find(|dep| which(dep).is_none())
}

/// `name` の実行ファイルを `$PATH` から探す（簡易 which）。
fn which(name: &str) -> Option<PathBuf> {
    // パス区切りを含む名前は PATH 探索せず、そのまま実行ファイルとして扱う。
    if name.contains('/') {
        let p = PathBuf::from(name);
        return is_executable(&p).then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

/// 実ファイルかつ実行ビットが立っているか（Unix）。
#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// 非 Unix では実ファイルの存在のみで判定する。
#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}
