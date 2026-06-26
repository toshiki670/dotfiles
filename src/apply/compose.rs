//! ファイル合成経路（§5.5）: dst=ファイルへ条件付き断片を `strategy` で重ねて書き出す。
//!
//! copy のツリー配置（dst=ディレクトリ）と異なり、generate と overlay 明示はここを通る。
//! 評価順の不変条件（§5.5）を実装する: ②overlay は宣言順に `when` 評価し満たす断片だけ採用、
//! ③`preserve = true` の既存 dst は最下層（土台）として断片の下に敷く。合成後の出力には
//! `locals` の `@@name@@` 注入を通す（§9。空マップなら素通り）。
//! 不変条件①（ユニット gate 先・false で短絡）は [`crate::apply`] が前段で担う。

use crate::apply::set_mode;
use crate::manifest::{Manifest, Overlay, Strategy};
use crate::{gate, generate, resolve, strategy};
use std::collections::BTreeMap;
use std::path::Path;

/// 1 単位（`dir`）を overlay 合成で `dst`（ファイル）へ生成・配置する。
/// 合成結果へ解決済み `locals` を注入してから書き出す（空なら注入なし）。
pub fn place(
    dir: &Path,
    dst: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
) -> Result<(), String> {
    let frags = resolve_fragments(dir, manifest)?;
    let bytes = combine(manifest, dst, &frags)?;
    let bytes = resolve::inject(&bytes, locals);

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(dst, &bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", dst.display()))?;
    set_mode(dst, manifest)?;
    Ok(())
}

/// 採用する断片列を解決する（不変条件②）。`preserve`（土台＝既存 dst）の配線は [`combine`]。
///
/// overlay 未記述時は生成方式の既定挙動（compose に来るのは generate のみ ＝ cmd 出力＋sibling）。
/// overlay 明示時は宣言順に `when` 評価し、満たす断片だけを採用する。
fn resolve_fragments(dir: &Path, manifest: &Manifest) -> Result<Vec<Vec<u8>>, String> {
    if manifest.overlay.is_empty() {
        return generate::default_fragments(dir, manifest);
    }

    // ②宣言順に when を評価し、満たす断片だけ採用。
    let mut frags = Vec::new();
    for ov in &manifest.overlay {
        if gate::when_satisfied(&ov.when) {
            frags.push(materialize(dir, ov)?);
        }
    }
    Ok(frags)
}

/// overlay 断片を実体化する（生成方式）: `src`（copy）/ `cmd`（generate）の択一。
fn materialize(dir: &Path, ov: &Overlay) -> Result<Vec<u8>, String> {
    if let Some(src) = &ov.src {
        let path = dir.join(src);
        std::fs::read(&path).map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))
    } else if !ov.cmd.is_empty() {
        generate::run_cmd(&ov.cmd)
    } else {
        Err("overlay は src / cmd のいずれかが必要です".to_string())
    }
}

/// `strategy` で断片を 1 ファイル分のバイト列へ合成する。
///
/// overlay 明示時は `strategy` を load 時に必須化済み（[`Manifest::validate`]）なので、`unwrap_or`
/// の暗黙 `concat` は overlay 未記述の generate 既定挙動だけに効く。`json-shallow` ＋ `preserve`
/// のときだけ既存 dst を読み、最下層の土台として断片の下に敷く（既存が無ければ土台なし）。
fn combine(manifest: &Manifest, dst: &Path, frags: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    match manifest.strategy.unwrap_or(Strategy::Concat) {
        Strategy::Concat => Ok(strategy::concat(frags)),
        Strategy::JsonShallow => {
            let base = if manifest.preserve {
                std::fs::read(dst).ok()
            } else {
                None
            };
            strategy::json_shallow(frags, base.as_deref())
        }
    }
}
