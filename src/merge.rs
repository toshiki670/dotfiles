//! `dotfiles apply` の merge 層: base（共有設定）へ既存ファイルのローカルキーを温存して書き出す。
//!
//! 設計書 §5（merge）/ §7（preserve）。対象は `~/.claude/settings.json` の1件のみ。
//! copy（実体コピー）/ generate（コマンド出力）と異なり、merge は **dst=ファイル**で、
//! 単位ディレクトリ直下の base JSON（`manifest.toml` 以外の唯一の実ファイル）を土台に、
//! 既存 dst から `preserve` キーだけを引き継ぐ **shallow merge**（トップレベルのみ）を行う。
//! 共有キー（hooks / statusLine / language 等）は base がそのまま勝ち、dotfiles が上書きする。
//!
//! 旧 chezmoi `modify_settings.json.tmpl` は `$local + $forced`（ローカル全キー温存）だったが、
//! 本層は設計書の明示 `preserve` allowlist に従い「base が土台、preserve キーのみ温存」とする
//! （preserve に無いローカル固有キーは温存されない）。

use crate::apply::{Outcome, set_mode};
use crate::discover::{self, MANIFEST};
use crate::manifest::Manifest;
use serde_json::{Map, Value};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// 1 単位（`dir`）を merge で `dst`（ファイル）へ配置する。
pub fn place(dir: &Path, dst: &Path, manifest: &Manifest) -> Result<Outcome, String> {
    // base ＝ 単位ディレクトリ直下の `manifest.toml` 以外の唯一の実ファイル。
    let base = read_json_object(&base_file(dir)?)?;

    // 既存 dst（未存在・空は {} 扱い。初回 apply 時など）から preserve キーを温存する。
    let existing = if dst.is_file() {
        read_json_object(dst)?
    } else {
        Map::new()
    };

    let merged = merge(base, &existing, &manifest.preserve);

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(&Value::Object(merged))
        .map_err(|e| format!("{}: JSON 直列化失敗: {e}", dst.display()))?;
    bytes.push(b'\n'); // POSIX テキストファイルとして末尾改行を補う。
    std::fs::write(dst, &bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", dst.display()))?;
    set_mode(dst, manifest)?;
    Ok(Outcome::Placed)
}

/// base を土台に、`existing` の `preserve` キーだけを上書きで温存する shallow merge。
///
/// base のキー（共有設定）はそのまま勝つ。`preserve` に挙げたトップレベルキーは、
/// `existing` に在ればその値で base を上書きする（ローカルが勝つ）。`existing` に無い
/// preserve キーは base のまま（base にも無ければ単に欠落）。値はトップレベル単位で
/// 丸ごと差し替える（深いマージはしない）。
fn merge(
    mut base: Map<String, Value>,
    existing: &Map<String, Value>,
    preserve: &[String],
) -> Map<String, Value> {
    for key in preserve {
        if let Some(value) = existing.get(key) {
            base.insert(key.clone(), value.clone());
        }
    }
    base
}

/// 単位ディレクトリ直下の base ファイル（`manifest.toml` 以外の実ファイル）を 1 つ特定する。
/// merge は base が 1 ファイルであることを要求する（0 個 / 複数はエラー）。
fn base_file(dir: &Path) -> Result<PathBuf, String> {
    let mut files: Vec<PathBuf> = discover::read_dir(dir)?
        .into_iter()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.file_name() != Some(OsStr::new(MANIFEST)))
        .collect();
    files.sort();
    match files.len() {
        1 => Ok(files.pop().unwrap()),
        0 => Err(format!(
            "{}: merge には base ファイルが必要です（{MANIFEST} 以外の実ファイルが無い）",
            dir.display(),
        )),
        n => Err(format!(
            "{}: merge の base は 1 ファイルのみ対応（{n} 個見つかった: {})",
            dir.display(),
            files
                .iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .collect::<Vec<_>>()
                .join(", "),
        )),
    }
}

/// JSON ファイルを読み、トップレベルが object であることを確かめて返す。
/// 空ファイルは `{}` 扱い（既存 dst が空内容のときに備える）。
fn read_json_object(path: &Path) -> Result<Map<String, Value>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("{}: JSON パース失敗: {e}", path.display()))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(format!(
            "{}: トップレベルが JSON object ではありません",
            path.display()
        )),
    }
}
