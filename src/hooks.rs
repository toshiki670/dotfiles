//! 配置後フック（#546）: ユニット配置後（after フェーズ）に、manifest が宣言した各フックを
//! **実行頻度（`frequency`）で分岐**して実行する**汎用エンジン**。
//!
//! ツール固有のロジックは binary に一切持たない。フックは manifest の `hooks` 属性が
//! `cmd`（argv・コマンド列）を**データ**として宣言し、本モジュールはそれを実行するだけ ―
//! [`crate::apply::generate`] の `cmd`（manifest のコマンドをデータとして実行）と同じ思想で、
//! 新ツールのフック追加に binary 変更・再コンパイルは要らない（configs と疎結合・スケールする）。
//! どのフックが macOS 専用か等の知識は manifest 側（ghostty の `os = "darwin"` ＋ コマンド本体）が
//! 持ち、エンジンは関知しない。
//!
//! 頻度で実行モデルが分かれる:
//! - **`onchange`**（既定）: onchange gate（[`crate::hooks::onchange`]）を通す。ユニットのソース
//!   ハッシュ＋コマンド内容が前回適用時と同じならスキップ、変化（初回・**コマンド変更**を含む）なら実行する。
//! - **`always`**: gate を通さず毎 apply 無条件に実行する（状態を読み書きしない）。反映対象が dotfiles
//!   管理外で随時変わる用途向けで、**コマンドの冪等性を前提**とする。
//!
//! いずれの頻度でも、プログラムが PATH に無い（未インストール）ときは skip してメッセージだけ出す
//! （`command -v` ガード相当を、ツール名を持たずに汎用化）。実際の spawn と `argv[0]` 解決は
//! [`mod@exec`] が担う。`onchange` はさらにハッシュを保存しないので、後で入れたら再実行される。トップレベル
//! `when`（ユニット gate, `deps` / `os`）が false のユニットは配置ごと skip されるため hooks も走らない
//! （＝ `when.os` でフックを分岐できる）。

mod exec;
pub(crate) mod onchange;

use crate::manifest::{Frequency, Hook};
use exec::{Exec, exec};
use onchange::State;
use std::path::Path;

/// ユニットが宣言した全 hooks を宣言順に、各フックの `frequency` で分岐して実行する
/// （[`crate::apply`] が配置後に呼ぶ）。onchange のソースハッシュは 1 回だけ計算して使い回す
/// （always だけのユニットでは無駄になるので計算しない）。
pub fn run_unit_hooks(
    unit_dir: &Path,
    unit_rel: &str,
    hooks: &[Hook],
    state: &mut State,
) -> Result<(), String> {
    if hooks.is_empty() {
        return Ok(());
    }
    // ソースハッシュは onchange 頻度のフックだけが要る。1 つでもあれば 1 回計算して使い回し、
    // always だけのユニットでは走査を丸ごと省く（反映対象が管理外＝ソースを見ても意味が無いため）。
    let source_hash = if hooks.iter().any(|h| h.frequency == Frequency::Onchange) {
        Some(onchange::hash_dir(unit_dir)?)
    } else {
        None
    };
    for hook in hooks {
        match hook.frequency {
            Frequency::Onchange => {
                // onchange が 1 つでもあれば上で source_hash は Some になっている（不変条件）。
                let source_hash = source_hash
                    .as_deref()
                    .expect("onchange フックがあれば source_hash は計算済み");
                run_onchange(&hook.cmd, unit_dir, unit_rel, source_hash, state)?;
            }
            Frequency::Always => run_always(&hook.cmd, unit_dir, unit_rel)?,
        }
    }
    Ok(())
}

/// 1 つの onchange フック（`frequency = "onchange"`）を onchange gate を通して実行する。
///
/// 状態キーは `<unit>::<コマンドの短ハッシュ>`。コマンド内容をキーに織り込むことで、manifest 上で
/// **コマンドを変えた場合も新しいキー＝再実行**になる（`manifest.toml` はソースハッシュ対象外なので、
/// これが無いとコマンド変更を取りこぼす）。値はユニットの**ソース**ハッシュ（中身の変化で再実行）。
fn run_onchange(
    argv: &[String],
    unit_dir: &Path,
    unit_rel: &str,
    source_hash: &str,
    state: &mut State,
) -> Result<(), String> {
    let Some(program) = argv.first() else {
        // manifest 検証で非空を保証済みだが防御的に弾く。
        return Err(format!("{unit_rel}: hook コマンドが空です"));
    };
    let label = argv.join(" ");
    let key = format!("{unit_rel}::{}", onchange::short_hash(&argv.join("\u{0}")));

    if state.get(&key) == Some(source_hash) {
        println!("hook: {label} ({unit_rel}) → skip (ソース不変)");
        return Ok(());
    }

    match exec(argv, unit_dir)? {
        Exec::Ran => {
            // 実行できたときだけ前回ハッシュを更新する（途中失敗時の取りこぼし防止に逐次保存）。
            state.set(&key, source_hash);
            state.save()?;
            println!("hook: {label} ({unit_rel}) → ran");
        }
        Exec::ProgramMissing => {
            // 未インストール → ハッシュ未保存（入れたら次回 apply で再実行）。
            println!("hook: {label} ({unit_rel}) → skip ({program} が PATH にない)");
        }
    }
    Ok(())
}

/// 1 つの always フック（`frequency = "always"`）を毎 apply 無条件に実行する。
///
/// onchange gate（[`State`] の読み書き）を通さず [`mod@exec`] を毎回呼ぶ。反映対象が dotfiles 管理外で
/// 随時変わる用途（copy/compose と同じ「常に再実行」）向けで、**コマンドが冪等であること**を前提とする
/// ― 毎 apply 無条件に走るため。未インストール（[`Exec::ProgramMissing`]）は onchange と同じく skip
/// 表示に留める（ハッシュを持たないので保存も無い）。
fn run_always(argv: &[String], unit_dir: &Path, unit_rel: &str) -> Result<(), String> {
    let Some(program) = argv.first() else {
        // manifest 検証で非空を保証済みだが防御的に弾く。
        return Err(format!("{unit_rel}: hook コマンドが空です"));
    };
    let label = argv.join(" ");
    match exec(argv, unit_dir)? {
        Exec::Ran => println!("hook: {label} ({unit_rel}) → ran (always)"),
        Exec::ProgramMissing => {
            println!("hook: {label} ({unit_rel}) → skip ({program} が PATH にない)");
        }
    }
    Ok(())
}
