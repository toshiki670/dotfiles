//! step 列の実行器: 内容（Content）を空から始め、宣言順に input（読む）→ output（書く）を畳む。
//!
//! `manifest.toml` の `[[steps]]` を解釈した [`crate::manifest::Steps`] を実行する。ツリー配置
//! （[`Steps::Tree`]）は step 列を持たないため、畳み込みを経ず単位ディレクトリを output へ再帰配置
//! する（[`crate::apply::copy`]）。パイプライン（[`Steps::Pipeline`]）は step 列を上から評価する:
//! 各 step は `when`（step スコープ gate）で採否を決め、採用された input は内容へ中身を畳み、
//! output は内容を宛先へ書く。
//!
//! バイト内容の畳み込みは unit レベルの `format` が内容種別を決め、各 input step 自身の `merge`
//! （値の一覧は [`crate::manifest`]）が「どう重ねるか」を選ぶ（[`fold_in`]） ― `fold_in` は unit 全体
//! ではなくその step の `merge` だけを見て畳み込み関数を選ぶ。最初の input は `merge` を持たない
//! （load 時の解釈が禁止）が、土台なし（`base = None`）から畳むといずれの畳み込み
//! 関数も断片そのもの（再直列化）を返す ― [`crate::apply::fold`] の各関数に共通する性質 ― ため、
//! optional / gate で先頭 input が実行時に飛んでも（宣言上の 2 番目が実際の最初になっても）結果は
//! 一定になる。
//!
//! 不変条件①（ユニット gate 先・false で短絡）は [`crate::apply`] が前段で担う（gate=false のユニットは
//! ここへ来ない）。

use super::gate::{self, GateState};
use super::{cmd, copy, fold, set_mode, write_if_changed};
use crate::locals::resolve;
use crate::manifest::{
    Format, InputStep, Manifest, Merge, OutputStep, Step, StepSource, Steps, resolve_output_path,
};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// 実行時の内容（manifest スキーマには現れないパイプライン内部型。[`crate`] 冒頭の用語集の
/// 「内容」に対応する）。
enum Content {
    /// まだ何も畳んでいない（初期状態・全 input が optional / gate で飛んだ状態）。
    Empty,
    /// バイト列の内容（input を `format` × `merge` で畳んだ結果）。
    Bytes(Vec<u8>),
}

/// 1 単位（`unit_dir`）の steps を実行する。`home` は `~` 展開先、`locals` は解決済み named value、
/// `gate_state` は step の `when`（状態 gate）評価に使う現在状態スナップショット。
pub fn run(
    unit_dir: &Path,
    home: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
    gate_state: &GateState,
) -> Result<(), String> {
    let (format, steps) = match &manifest.steps {
        Steps::Tree { output } => {
            return copy::place(
                unit_dir,
                &resolve_output_path(home, output),
                manifest,
                locals,
            );
        }
        Steps::Pipeline { format, steps } => (*format, steps),
    };
    let mut content = Content::Empty;
    for step in steps {
        match step {
            Step::Input(input) => {
                // step gate: 満たさなければこの step を飛ばす（内容は不変）。
                if !gate::when_satisfied(&input.when, gate_state) {
                    continue;
                }
                apply_input(&mut content, unit_dir, home, format, input)?;
            }
            Step::Output(output) => {
                if !gate::when_satisfied(&output.when, gate_state) {
                    continue;
                }
                apply_output(&content, home, manifest, locals, output)?;
            }
        }
    }
    Ok(())
}

/// input step: 読んだ中身を内容へ畳む（`format` × この step の `merge` で駆動）。
/// optional なパス input が不在なら内容を触らず飛ばす（次の input が土台になる）。
fn apply_input(
    content: &mut Content,
    unit_dir: &Path,
    home: &Path,
    format: Option<Format>,
    input: &InputStep,
) -> Result<(), String> {
    match &input.source {
        StepSource::Path(p) => {
            let path = resolve_input_path(unit_dir, home, p);
            match std::fs::read(&path) {
                // パースエラーには input パスのラベルを添える（どの input が壊れたか）。
                Ok(bytes) => {
                    fold_in(content, format, input.merge, &bytes)
                        .map_err(|e| format!("{p}: {e}"))?;
                }
                // optional な不在は内容を触らず飛ばす（既定は「無ければエラー」）。
                Err(e) if e.kind() == io::ErrorKind::NotFound && input.optional => {}
                Err(e) => return Err(format!("{}: 読み込み失敗: {e}", path.display())),
            }
        }
        StepSource::Cmd(c) => {
            let bytes = cmd::run(&c.cmd)?;
            // パースエラーには cmd argv のラベルを添える。
            fold_in(content, format, input.merge, &bytes)
                .map_err(|e| format!("{}: {e}", c.cmd.join(" ")))?;
        }
    }
    Ok(())
}

/// output step: 内容を宛先へ書く。パス output はファイルを配置、cmd output は内容を
/// 標準入力へ渡す。
fn apply_output(
    content: &Content,
    home: &Path,
    manifest: &Manifest,
    locals: &BTreeMap<String, String>,
    output: &OutputStep,
) -> Result<(), String> {
    let Content::Bytes(bytes) = content else {
        return Err(
            "output に到達しましたが内容が空です（生成された中身がありません）".to_string(),
        );
    };
    match &output.dest {
        StepSource::Path(p) => {
            let injected = resolve::inject(bytes, locals);
            let path = resolve_output_path(home, p);
            write_if_changed(&path, &injected)?;
            // 書き込みを省略しても mode は毎回再適用する（属性変更が反映されるように）。
            set_mode(&path, manifest)
        }
        StepSource::Cmd(c) => {
            // cmd output は毎 apply 実行し、合成済みの内容を標準入力へ渡す（冪等契約）。
            cmd::run_piped(&c.cmd, &resolve::inject(bytes, locals))
        }
    }
}

/// 読んだ新しい中身（`bytes`）を内容へ畳む。畳み方は unit の `format`（内容種別）と、この step 自身の
/// `merge`（重ね方。最初の input は常に `None`）の組で決まる ― モジュール doc 参照。
///
/// 内容が空（`Empty`）なら土台なし（`base = None`）で最初の中身を畳む ― [`fold::text::concat`] は
/// 土台が無ければ境目の改行を補わず、[`fold::json`] / [`fold::plist`] の `shallow` / `deep` は
/// いずれも最初の断片そのものを返すため、最初の input（`merge = None`）は中身そのまま（再直列化）に
/// なる。`format = None`（merge を使わないユニット）は中身をそのまま内容にする。
fn fold_in(
    content: &mut Content,
    format: Option<Format>,
    merge: Option<Merge>,
    bytes: &[u8],
) -> Result<(), String> {
    let base = match content {
        Content::Bytes(b) => Some(b.as_slice()),
        Content::Empty => None,
    };
    let merged = match (format, merge) {
        (None, _) => bytes.to_vec(),
        (Some(Format::Text), _) => fold::text::concat(base, bytes),
        (Some(Format::Json), Some(Merge::Deep)) => fold::json::deep(base, bytes)?,
        (Some(Format::Json), _) => fold::json::shallow(base, bytes)?,
        (Some(Format::Plist), Some(Merge::Deep)) => fold::plist::deep(base, bytes)?,
        (Some(Format::Plist), _) => fold::plist::shallow(base, bytes)?,
    };
    *content = Content::Bytes(merged);
    Ok(())
}

/// input パスを解決する: `~` 起点は `home` に展開、それ以外は単位相対（`unit_dir` に join）。
/// 表記は load 時の解釈（[`crate::manifest`]）が保証済み（`~` / `~/...` / 単位相対）。
fn resolve_input_path(unit_dir: &Path, home: &Path, p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        home.join(rest)
    } else if p == "~" {
        home.to_path_buf()
    } else {
        unit_dir.join(p)
    }
}
