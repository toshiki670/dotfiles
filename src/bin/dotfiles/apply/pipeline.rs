//! step 列の実行器: 文書 D を空から始め、宣言順に input（読む）→ output（書く）を畳む。
//!
//! `manifest.toml` の `[[steps]]` を上から評価する。各 step は `when`（step スコープ gate）で採否を
//! 決め、採用された input は D へ内容を畳み、output は D を宛先へ書く。ツリー input（`input = "."`）は
//! D を「ファイルツリー」にし、パス output で相対構造を保って再帰配置する（[`crate::apply::copy`]）。
//!
//! **設計の核**: manifest の `merge` は load 時の整合検証のための注釈で、実行時の畳み込みは分岐させない。
//! 畳み込みは unit レベルの単一値 `format` だけで駆動する（[`fold_in`]）。`format`-駆動の畳み込みを
//! 全 input に一様に適用すると「最初の input は置換・2 つ目以降は merge」と byte 単位で一致する ―
//! [`crate::apply::strategy`] の `concat` / `json_shallow` / `plist_shallow` はいずれも空の土台から
//! 最初の断片を畳むと断片そのもの（再直列化）を返す incremental な後勝ち fold だから。これにより、
//! optional / gate で先頭 input が実行時に飛んでも（宣言上の 2 番目が実際の最初になっても）結果は
//! 一定になる。
//!
//! 不変条件①（ユニット gate 先・false で短絡）は [`crate::apply`] が前段で担う（gate=false のユニットは
//! ここへ来ない）。

use super::gate::{self, GateState};
use super::{cmd, copy, set_mode, strategy};
use crate::locals::resolve;
use crate::manifest::{Format, Manifest, StepSource};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// 実行時の文書 D（manifest スキーマには現れないパイプライン内部型）。
enum Document {
    /// まだ何も畳んでいない（初期状態・全 input が optional / gate で飛んだ状態）。
    Empty,
    /// バイト文書（input を `format` で畳んだ結果）。
    Bytes(Vec<u8>),
    /// 単位ディレクトリツリー（`input = "."`）。パス output で再帰配置する。
    Tree,
}

/// 1 単位（`unit_dir`）の step 列を実行する。`home` は `~` 展開先、`locals` は解決済み named value、
/// `gate_state` は step の `when`（状態 gate）評価に使う現在状態スナップショット。
pub fn run(
    unit_dir: &Path,
    home: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
    gate_state: &GateState,
) -> Result<(), String> {
    let mut doc = Document::Empty;
    for step in &manifest.steps {
        // step gate: 満たさなければこの step を飛ばす（D は不変）。
        if !gate::when_satisfied(&step.when, gate_state) {
            continue;
        }
        if let Some(input) = &step.input {
            apply_input(
                &mut doc,
                unit_dir,
                home,
                manifest.format,
                step.optional,
                input,
            )?;
        } else if let Some(output) = &step.output {
            apply_output(&doc, unit_dir, home, manifest, locals, output)?;
        }
    }
    Ok(())
}

/// input step: 内容を D へ畳む（`format` 駆動）。ツリー input は D をツリーにする。
/// optional なパス input が不在なら D を触らず飛ばす（次の input が土台になる）。
fn apply_input(
    doc: &mut Document,
    unit_dir: &Path,
    home: &Path,
    format: Option<Format>,
    optional: bool,
    input: &StepSource,
) -> Result<(), String> {
    match input {
        StepSource::Path(p) if p == "." => *doc = Document::Tree,
        StepSource::Path(p) => {
            let path = resolve_input_path(unit_dir, home, p);
            match std::fs::read(&path) {
                Ok(bytes) => fold_in(doc, format, bytes)?,
                // optional な不在は D を触らず飛ばす（既定は「無ければエラー」）。
                Err(e) if e.kind() == io::ErrorKind::NotFound && optional => {}
                Err(e) => return Err(format!("{}: 読み込み失敗: {e}", path.display())),
            }
        }
        StepSource::Cmd(c) => fold_in(doc, format, cmd::run(&c.cmd)?)?,
    }
    Ok(())
}

/// output step: D を宛先へ書く。パス output はファイル/ツリーを配置、cmd output は D を標準入力へ渡す。
fn apply_output(
    doc: &Document,
    unit_dir: &Path,
    home: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
    output: &StepSource,
) -> Result<(), String> {
    match (output, doc) {
        (StepSource::Path(p), Document::Tree) => {
            copy::place(unit_dir, &resolve_output_path(home, p), manifest, locals)
        }
        (StepSource::Path(p), Document::Bytes(bytes)) => {
            let injected = resolve::inject(bytes, locals);
            let path = resolve_output_path(home, p);
            write_if_changed(&path, &injected)?;
            // 書き込みを省略しても mode は毎回再適用する（属性変更が反映されるように）。
            set_mode(&path, manifest)
        }
        (StepSource::Cmd(c), Document::Bytes(bytes)) => {
            // cmd output は毎 apply 実行し、合成済み D を標準入力へ渡す（冪等契約）。
            cmd::run_piped(&c.cmd, &resolve::inject(bytes, locals))
        }
        (_, Document::Empty) => {
            Err("output に到達しましたが D が空です（生成された内容がありません）".to_string())
        }
        (StepSource::Cmd(_), Document::Tree) => {
            unreachable!("validated: ツリー output はパス（cmd 不可）")
        }
    }
}

/// 新しい内容を D へ畳む。畳み方は unit の `format` だけで決まる（`merge` は実行時に見ない）。
///
/// D が空（`Empty`）なら土台なしで最初の内容を畳む ― `concat` は空の out に対し境目の改行を補わず、
/// `json_shallow` / `plist_shallow` は `base = None` で最初の断片そのものを返すため、いずれも最初の
/// input は内容そのまま（再直列化）になる。`format = None`（merge を使わないユニット）は内容をそのまま
/// D にする。
fn fold_in(doc: &mut Document, format: Option<Format>, content: Vec<u8>) -> Result<(), String> {
    let base = match doc {
        Document::Bytes(b) => Some(b.as_slice()),
        _ => None,
    };
    let merged = match format {
        None => content,
        Some(Format::Text) => {
            strategy::concat(&[base.map(<[u8]>::to_vec).unwrap_or_default(), content])
        }
        Some(Format::Json) => strategy::json_shallow(&[content], base)?,
        Some(Format::Plist) => strategy::plist_shallow(&[content], base)?,
    };
    *doc = Document::Bytes(merged);
    Ok(())
}

/// 現在内容と一致すれば書き込みを省略する（冪等最適化）。親ディレクトリは作成する。
fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if std::fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))
}

/// input パスを解決する: `~` 起点は `home` に展開、それ以外は単位相対（`unit_dir` に join）。
/// 表記は [`crate::manifest`] の `validate` が保証済み（`~` / `~/...` / 単位相対）。
fn resolve_input_path(unit_dir: &Path, home: &Path, p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        home.join(rest)
    } else if p == "~" {
        home.to_path_buf()
    } else {
        unit_dir.join(p)
    }
}

/// output パスを解決する: 常に `~` 起点（`validate` が `~` / `~/...` のみを保証）。
fn resolve_output_path(home: &Path, p: &str) -> PathBuf {
    p.strip_prefix("~/")
        .map_or_else(|| home.to_path_buf(), |rest| home.join(rest))
}
