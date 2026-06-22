//! `dotfiles apply`：固定ソース `configs/` を走査し配置を実行する。
//!
//! 走査（manifest 発見・再帰委譲）は [`crate::discover`]。各単位は §5.5 の評価順に従う:
//! ①**ユニット gate（`deps` / `os`）を最初に評価し false なら短絡**（[`crate::gate`]、dst を
//! 一切触らず skip）。生き残った単位は dst 種別で配置経路が分かれる ―
//! dst=ディレクトリの copy は [`crate::copy`]（ツリー配置）、dst=ファイルの generate /
//! overlay 明示は [`crate::compose`]（②宣言順 overlay ③preserve=既存 dst を土台に敷く合成）。
//! 配置の直前に `locals`（named value）を解決し（[`crate::resolve`]）、配置ファイルの `@@name@@` を
//! 注入する（§9）。配置成功後に `hooks`（onchange フック）を、ユニットのソースハッシュが前回適用時
//! から変わっていれば実行する（[`crate::hooks`] / [`crate::onchange`]、§13）。ユニット gate が false の
//! ときは配置前に短絡 return するため、その hooks も走らない（＝ os 属性でフックを分岐できる）。
//! 本モジュールはオーケストレーションと、両経路が共有する小道具（`~` 展開・パーミッション適用）を持つ。

use crate::discover::{self, MANIFEST, Unit};
use crate::manifest::{Kind, Manifest, Strategy};
use crate::onchange::State as HookState;
use crate::store::Store;
use crate::{compose, copy, gate, hooks, prompt, resolve};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `source`（= `configs/`）配下を走査し、各 manifest の配置を実行する。
/// `home` は dst の `~` 展開先。`locals` の取得・注入に使う named value ストアと、`hooks` の onchange
/// 状態（[`HookState`]）は開始時に1回ロードし、各単位で逐次更新する。
pub fn run(source: &Path, home: &Path) -> Result<(), String> {
    let units = discover::collect(source)?;
    if units.is_empty() {
        println!(
            "apply: 対象なし（{} に manifest.toml がない）",
            source.display()
        );
        return Ok(());
    }

    let mut store = Store::load(home)?;
    let mut hook_state = HookState::load(home);
    for unit in &units {
        apply_unit(unit, home, &mut store, &mut hook_state)?;
    }
    Ok(())
}

/// 1 単位を評価順（§5.5）に従って配置し、結果を 1 行で表示する。配置成功後、ユニットが宣言した
/// `hooks` を onchange gate を通して実行する（§13）。
fn apply_unit(
    unit: &Unit,
    home: &Path,
    store: &mut Store,
    hook_state: &mut HookState,
) -> Result<(), String> {
    let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
    let dst = expand_home(&manifest.dst, home);
    let name = unit.rel.to_string_lossy();

    // ①ユニット gate を最初に評価し、満たさなければユニット全体を skip（dst も hooks も触らない）。
    if let Some(reason) = gate::unit_skip_reason(&manifest) {
        println!("apply: {name} → skip ({reason})");
        return Ok(());
    }

    // `locals` 宣言があれば配置前に解決する（TTY=対話 / 非TTY=警告のみ）。宣言が無ければ空マップ
    // ＝注入経路を素通りし、無関係ファイルの `@@…@@` を巻き込まない（§9.1）。
    let locals = if manifest.locals.is_empty() {
        BTreeMap::new()
    } else {
        resolve::fill(&manifest, store, prompt::is_tty())?
    };

    let label = placement_label(&manifest);
    if uses_compose(&manifest) {
        compose::place(&unit.dir, &dst, &manifest, &locals)?;
    } else {
        copy::place(&unit.dir, &dst, &manifest, &locals)?;
    }
    println!("apply: {name} → {} ({label})", manifest.dst);

    // 配置後（after フェーズ）に onchange フックを走らせる。ソースが前回適用時と同じなら skip。
    for hook in &manifest.hooks {
        hooks::run_if_changed(hook, &unit.dir, &name, home, hook_state)?;
    }
    Ok(())
}

/// ファイル合成経路（[`crate::compose`]）を通すか。overlay 明示、または dst=ファイルの
/// generate はファイル合成。それ以外（overlay 無しの copy）は copy ツリー配置。
fn uses_compose(manifest: &Manifest) -> bool {
    !manifest.overlay.is_empty() || manifest.kind == Kind::Generate
}

/// 表示用の配置ラベル（apply の 1 行出力）。overlay 明示は strategy を併記する。
fn placement_label(manifest: &Manifest) -> &'static str {
    if !manifest.overlay.is_empty() {
        return match manifest.strategy {
            Some(Strategy::JsonShallow) => "overlay/json-shallow",
            Some(Strategy::Concat) => "overlay/concat",
            None => "overlay",
        };
    }
    match manifest.kind {
        Kind::Copy => "copy",
        Kind::Generate => "generate",
    }
}

/// 配置済みファイルへ manifest のパーミッションを適用する（Unix のみ）。
/// copy / compose 両経路が共有する。
#[cfg(unix)]
pub fn set_mode(dst: &Path, manifest: &Manifest) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dst, std::fs::Permissions::from_mode(manifest.mode()))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", dst.display()))
}

/// 非 Unix では no-op（パーミッションモデルが異なるため）。
#[cfg(not(unix))]
pub fn set_mode(_dst: &Path, _manifest: &Manifest) -> Result<(), String> {
    Ok(())
}

/// dst の `~` / `~/...` を `home` に展開する。
/// `$XDG_*` 等の正規化は設計書 §14 で確定。
fn expand_home(dst: &str, home: &Path) -> PathBuf {
    if let Some(rest) = dst.strip_prefix("~/") {
        home.join(rest)
    } else if dst == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(dst)
    }
}
