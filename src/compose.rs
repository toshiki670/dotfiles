//! ファイル合成経路（§5.5）: dst=ファイルへ条件付き断片を `strategy` で重ねて書き出す。
//!
//! copy のツリー配置（dst=ディレクトリ）と異なり、generate と overlay 明示はここを通る。
//! 評価順の不変条件（§5.5）を実装する: ②overlay は宣言順に `when` 評価し満たす断片だけ採用、
//! ③`preserve`（既存 dst を読む built-in overlay）は宣言位置に関わらず最後に重ねる。
//! 不変条件①（ユニット gate 先・false で短絡）は [`crate::apply`] が前段で担う。

use crate::apply::set_mode;
use crate::manifest::{Manifest, Overlay, Strategy};
use crate::{gate, generate, strategy};
use std::path::Path;

/// 1 単位（`dir`）を overlay 合成で `dst`（ファイル）へ生成・配置する。
pub fn place(dir: &Path, dst: &Path, manifest: &Manifest) -> Result<(), String> {
    let (frags, preserve_keys) = resolve_fragments(dir, manifest)?;
    let bytes = combine(manifest, dst, &frags, &preserve_keys)?;

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(dst, &bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", dst.display()))?;
    set_mode(dst, manifest)?;
    Ok(())
}

/// 採用する断片列と preserve キーを解決する（不変条件②③）。
///
/// overlay 未記述時は生成方式の既定挙動（compose に来るのは generate のみ ＝ cmd 出力＋sibling）。
/// overlay 明示時は preserve 以外を宣言順に `when` 評価し、preserve overlay は最後へ畳む。
fn resolve_fragments(
    dir: &Path,
    manifest: &Manifest,
) -> Result<(Vec<Vec<u8>>, Vec<String>), String> {
    if manifest.overlay.is_empty() {
        return Ok((generate::default_fragments(dir, manifest)?, Vec::new()));
    }

    // ②宣言順に when を評価し、満たす断片だけ採用（preserve 以外）。
    let mut frags = Vec::new();
    for ov in manifest.overlay.iter().filter(|o| !o.is_preserve()) {
        if gate::when_satisfied(&ov.when) {
            frags.push(materialize(dir, ov)?);
        }
    }
    // ③preserve（既存 dst を読む built-in overlay）は宣言位置に関わらず最後に重ねる。
    let mut preserve_keys = Vec::new();
    for ov in manifest.overlay.iter().filter(|o| o.is_preserve()) {
        if gate::when_satisfied(&ov.when) {
            preserve_keys.extend(ov.preserve.iter().cloned());
        }
    }
    Ok((frags, preserve_keys))
}

/// overlay 断片を実体化する（生成方式）: `src`（copy）/ `cmd`（generate）の択一。
fn materialize(dir: &Path, ov: &Overlay) -> Result<Vec<u8>, String> {
    if let Some(src) = &ov.src {
        let path = dir.join(src);
        std::fs::read(&path).map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))
    } else if !ov.cmd.is_empty() {
        generate::run_cmd(&ov.cmd)
    } else {
        Err("overlay は src / cmd / preserve のいずれかが必要です".to_string())
    }
}

/// `strategy` で断片を 1 ファイル分のバイト列へ合成する。
///
/// generate 既定（`strategy` 省略）は `concat`。`json-shallow` のときだけ、preserve があれば
/// 既存 dst を読み最後に重ねる（既存が無ければ温存なし）。
fn combine(
    manifest: &Manifest,
    dst: &Path,
    frags: &[Vec<u8>],
    preserve_keys: &[String],
) -> Result<Vec<u8>, String> {
    match manifest.strategy.unwrap_or(Strategy::Concat) {
        Strategy::Concat => Ok(strategy::concat(frags)),
        Strategy::JsonShallow => {
            let existing = if preserve_keys.is_empty() {
                None
            } else {
                std::fs::read(dst).ok()
            };
            strategy::json_shallow(frags, preserve_keys, existing.as_deref())
        }
    }
}
