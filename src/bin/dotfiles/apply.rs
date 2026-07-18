//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する。
//!
//! 走査（manifest 発見・再帰委譲）は [`crate::discover`]。各単位は次の評価順に従う:
//! ①**トップレベル `when`（ユニット gate）を最初に評価し false なら短絡**（[`crate::apply::gate`]、
//! 配置を一切触らず skip）。生き残った単位は `[[steps]]` パイプライン（[`crate::apply::pipeline`]）で
//! 実行する ― 内容を空から始め、宣言順に input（読む）→ output（書く）を畳む。ツリー input
//! （`input = "."`）は [`crate::apply::copy`] で単位ディレクトリを再帰配置し（宣言されていれば末尾の
//! output.cmd を配置後に実行）、バイト内容の合成は [`crate::apply::fold`]、cmd 実行は
//! [`crate::apply::cmd`] が担う。配置の直前に `locals`（named value）を解決し
//! （[`crate::locals::resolve`]）、配置ファイルの `@@name@@` を注入する。
//! 本モジュールはオーケストレーションと、各配置経路が共有する小道具（パーミッション適用・冪等書き込み）
//! を持つ。
//!
//! 配置は実体の書き出し（copy）で、symlink は採用しない。`cargo install --git` で配布された
//! バイナリは埋め込みソースだけで配置できる必要があり、symlink はリンク先の実体（リポジトリの
//! clone 常設）を要求するため。編集の即時反映は捨て、反映は `dotfiles apply` の再実行で行う。
//!
//! gate=false は「配置しない」であって即「撤去する」ではない。step の脱落は、配置先が毎 apply
//! 再合成されるため次の apply で結果から消える。ユニット全体・配置先そのものの取り残し（ユニット
//! gate flip・ユニット削除・ツリーファイル削除）は既定では残ったまま（[`crate::prune`] が期待
//! 配置集合を毎回 snapshot へ記録するだけ）で、`force = true`（`dotfiles apply --force`）を渡した
//! ときだけ実際に退避する（#521）。

pub(crate) mod cmd;
pub(crate) mod copy;
mod fold;
pub(crate) mod gate;
pub(crate) mod pipeline;

use crate::discover::{self, MANIFEST, Unit};
use crate::locals::store::Store;
use crate::locals::{prompt, resolve};
use crate::manifest::Manifest;
use crate::placements;
use crate::prune;
use std::collections::BTreeMap;
use std::path::Path;

/// `source`（= `configs/`）配下を走査し、各 manifest の配置を実行する。
/// `home` は `~` 展開先。`locals` の取得・注入に使う named value ストアは開始時に 1 回ロードし、
/// 各単位で逐次更新する。`force` は #521 の実削除 opt-in（`dotfiles apply --force`）。
pub fn run(source: &Path, home: &Path, force: bool) -> Result<(), String> {
    let units = discover::collect(source)?;
    if units.is_empty() {
        println!(
            "apply: 対象なし（{} に manifest.toml がない）",
            source.display()
        );
        return Ok(());
    }

    let mut store = Store::load(home)?;
    // 状態駆動 gate（profile）の現在状態は開始時に 1 回だけ解決し、全ユニット・全 step で共有する。
    let gate_state = gate::GateState::load(home)?;
    for unit in &units {
        apply_unit(unit, home, &mut store, &gate_state)?;
    }

    // 今回の期待配置集合（gate 評価後）を snapshot へ記録する。既定は union（縮めない・裏方の記録で
    // 表には出さない）、force のときだけ stale 候補を退避して snapshot をリセットする（#521）。
    let current =
        placements::expected_gated(source, home, &|w| gate::when_satisfied(w, &gate_state))?;
    if force {
        let moved = prune::commit(home, &current)?;
        if moved.is_empty() {
            println!("apply: prune → 不要な配置はありません");
        }
        for path in &moved {
            println!("apply: prune → {} を退避しました", path.display());
        }
    } else {
        prune::union(home, &current)?;
    }
    Ok(())
}

/// 1 単位を評価順に従って配置し、結果を 1 行で表示する。
fn apply_unit(
    unit: &Unit,
    home: &Path,
    store: &mut Store,
    gate_state: &gate::GateState,
) -> Result<(), String> {
    let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
    let name = unit.rel.to_string_lossy();

    // ①トップレベル when（ユニット gate）を最初に評価し、満たさなければユニット全体を skip（配置を一切行わない）。
    if let Some(reason) = gate::unit_skip_reason(&manifest, gate_state) {
        println!("apply: {name} → skip ({reason})");
        return Ok(());
    }

    // `locals` 宣言があれば配置前に解決する（TTY=対話 / 非TTY=警告のみ）。宣言が無ければ空マップ
    // ＝注入経路を何もせず通過し、無関係ファイルの `@@…@@` を巻き込まない。
    let locals = if manifest.locals.is_empty() {
        BTreeMap::new()
    } else {
        resolve::fill(&manifest, store, prompt::is_tty())?
    };

    // パイプラインのエラーにはユニット名を前置する（32 ユニットのどれで壊れたか即座に分かるように）。
    pipeline::run(&unit.dir, home, &manifest, &locals, gate_state)
        .map_err(|e| format!("{name}: {e}"))?;
    // 宛先表記は最初のパス output（`~/...`）。cmd output だけの `stats` は `(cmd)` を出すが、`stats` は
    // `when.deps = ["defaults"]` で常に skip される（`real_configs` の空 PATH でも同様）ため、`(cmd)` は
    // `real_configs` の stdout パースには現れない ― とはいえ将来 manifest が変わっても壊れないよう明示。
    println!(
        "apply: {name} → {} ({})",
        manifest.display_dst(),
        manifest.summary()
    );
    Ok(())
}

/// 配置済みファイルへ manifest のパーミッションを適用する（Unix のみ）。
/// パイプラインの各配置経路（copy / パス output）が `super::set_mode` で共有する。
#[cfg(unix)]
fn set_mode(dst: &Path, manifest: &Manifest) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dst, std::fs::Permissions::from_mode(manifest.mode()))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", dst.display()))
}

/// 非 Unix では no-op（パーミッションモデルが異なるため）。
#[cfg(not(unix))]
fn set_mode(_dst: &Path, _manifest: &Manifest) -> Result<(), String> {
    Ok(())
}

/// 現在内容と一致すれば書き込みを省略する（冪等最適化）。親ディレクトリは作成する。
/// ツリー配置（[`crate::apply::copy`]）とパス output（[`crate::apply::pipeline`]）が
/// `super::write_if_changed` で共有し、byte-identical な再 apply で mtime を無用に更新しない
/// （config を監視するツールの誤リロードを避ける）。
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
