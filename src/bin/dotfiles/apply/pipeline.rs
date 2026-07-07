//! step 列の実行器: 内容（Content）を空から始め、宣言順に input（読む）→ output（書く）を畳む。
//!
//! `manifest.toml` の `[[steps]]` を上から評価する。各 step は `when`（step スコープ gate）で採否を
//! 決め、採用された input は内容へ中身を畳み、output は内容を宛先へ書く。ツリー input（`input = "."`）は
//! 内容を「ファイルツリー」にし、パス output で相対構造を保って再帰配置する（[`crate::apply::copy`]）。
//!
//! **slice 1 の性質（`format` 一様駆動）**: 本スライスでは manifest の `merge` は load 時の整合検証の
//! ための注釈にすぎず、実行時の畳み込みは unit レベルの単一値 `format` だけで駆動する（[`fold_in`]）。
//! `format`-駆動の畳み込みを全 input に一様に適用すると「最初の input は置換・2 つ目以降は merge」と
//! byte 単位で一致する ― [`crate::apply::fold`] の `concat` / `json_shallow` / `plist_shallow` はいずれも
//! 空の土台（`base = None`）から最初の断片を畳むと断片そのもの（再直列化）を返す incremental な後勝ち
//! fold だから。これにより、optional / gate で先頭 input が実行時に飛んでも（宣言上の 2 番目が実際の
//! 最初になっても）結果は一定になる。
//!
//! この一様さは、slice 1 で `format` と `merge` が 1:1 に対応する（json/plist→shallow・text→append）
//! から成り立つ暫定的な性質。slice 2 で `merge = "deep"` が入ると `format = "json"` が shallow / deep の
//! どちらとも対になり得るため、`fold_in` は per-step の `merge` を実行時に見る必要が生じ、この一様さは
//! 崩れる（#554 / #588）。
//!
//! 不変条件①（ユニット gate 先・false で短絡）は [`crate::apply`] が前段で担う（gate=false のユニットは
//! ここへ来ない）。

use super::gate::{self, GateState};
use super::{cmd, copy, fold, set_mode};
use crate::locals::resolve;
use crate::manifest::{Format, Manifest, StepSource};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// 実行時の内容（manifest スキーマには現れないパイプライン内部型）。ユビキタス言語では単に
/// 「内容」と呼び、rustdoc・issue の議論でも同じ語を使う。
enum Content {
    /// まだ何も畳んでいない（初期状態・全 input が optional / gate で飛んだ状態）。
    Empty,
    /// バイト列の内容（input を `format` で畳んだ結果）。
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
    let mut content = Content::Empty;
    for step in &manifest.steps {
        // step gate: 満たさなければこの step を飛ばす（内容は不変）。
        if !gate::when_satisfied(&step.when, gate_state) {
            continue;
        }
        if let Some(input) = &step.input {
            apply_input(
                &mut content,
                unit_dir,
                home,
                manifest.format,
                step.optional,
                input,
            )?;
        } else if let Some(output) = &step.output {
            apply_output(&content, unit_dir, home, manifest, locals, output)?;
        }
    }
    Ok(())
}

/// input step: 読んだ中身を内容へ畳む（`format` 駆動）。ツリー input は内容をツリーにする。
/// optional なパス input が不在なら内容を触らず飛ばす（次の input が土台になる）。
fn apply_input(
    content: &mut Content,
    unit_dir: &Path,
    home: &Path,
    format: Option<Format>,
    optional: bool,
    input: &StepSource,
) -> Result<(), String> {
    match input {
        StepSource::Path(p) if p == "." => *content = Content::Tree,
        StepSource::Path(p) => {
            let path = resolve_input_path(unit_dir, home, p);
            match std::fs::read(&path) {
                // パースエラーには input パスのラベルを添える（どの input が壊れたか）。
                Ok(bytes) => fold_in(content, format, &bytes).map_err(|e| format!("{p}: {e}"))?,
                // optional な不在は内容を触らず飛ばす（既定は「無ければエラー」）。
                Err(e) if e.kind() == io::ErrorKind::NotFound && optional => {}
                Err(e) => return Err(format!("{}: 読み込み失敗: {e}", path.display())),
            }
        }
        StepSource::Cmd(c) => {
            let bytes = cmd::run(&c.cmd)?;
            // パースエラーには cmd argv のラベルを添える。
            fold_in(content, format, &bytes).map_err(|e| format!("{}: {e}", c.cmd.join(" ")))?;
        }
    }
    Ok(())
}

/// output step: 内容を宛先へ書く。パス output はファイル/ツリーを配置、cmd output は内容を
/// 標準入力へ渡す。
fn apply_output(
    content: &Content,
    unit_dir: &Path,
    home: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
    output: &StepSource,
) -> Result<(), String> {
    match (output, content) {
        (StepSource::Path(p), Content::Tree) => {
            copy::place(unit_dir, &resolve_output_path(home, p), manifest, locals)
        }
        (StepSource::Path(p), Content::Bytes(bytes)) => {
            let injected = resolve::inject(bytes, locals);
            let path = resolve_output_path(home, p);
            write_if_changed(&path, &injected)?;
            // 書き込みを省略しても mode は毎回再適用する（属性変更が反映されるように）。
            set_mode(&path, manifest)
        }
        (StepSource::Cmd(c), Content::Bytes(bytes)) => {
            // cmd output は毎 apply 実行し、合成済みの内容を標準入力へ渡す（冪等契約）。
            cmd::run_piped(&c.cmd, &resolve::inject(bytes, locals))
        }
        (_, Content::Empty) => {
            Err("output に到達しましたが内容が空です（生成された中身がありません）".to_string())
        }
        (StepSource::Cmd(_), Content::Tree) => {
            unreachable!("validated: ツリー output はパス（cmd 不可）")
        }
    }
}

/// 読んだ新しい中身（`bytes`）を内容へ畳む。畳み方は unit の `format` だけで決まる
/// （slice 1 では `merge` を実行時に見ない ― モジュール doc 参照）。
///
/// 内容が空（`Empty`）なら土台なし（`base = None`）で最初の中身を畳む ― [`fold::concat`] は
/// 土台が無ければ境目の改行を補わず、[`fold::json_shallow`] / [`fold::plist_shallow`] は最初の断片
/// そのものを返すため、いずれも最初の input は中身そのまま（再直列化）になる。`format = None`
/// （merge を使わないユニット）は中身をそのまま内容にする。
fn fold_in(content: &mut Content, format: Option<Format>, bytes: &[u8]) -> Result<(), String> {
    let base = match content {
        Content::Bytes(b) => Some(b.as_slice()),
        _ => None,
    };
    let merged = match format {
        None => bytes.to_vec(),
        Some(Format::Text) => fold::concat(base, bytes),
        Some(Format::Json) => fold::json_shallow(base, bytes)?,
        Some(Format::Plist) => fold::plist_shallow(base, bytes)?,
    };
    *content = Content::Bytes(merged);
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
