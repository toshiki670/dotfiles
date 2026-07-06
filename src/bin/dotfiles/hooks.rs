//! 配置後フック: ユニット配置後（after フェーズ）に、manifest が宣言した各フックを onchange gate を
//! 通して実行する**汎用エンジン**。
//!
//! ツール固有のロジックは binary に一切持たない。フックは manifest の `hooks` 属性が
//! `cmd`（argv・コマンド列）を**データ**として宣言し、本モジュールはそれを実行するだけ ―
//! step の `cmd`（manifest のコマンドをデータとして実行）と同じ思想で、新ツールのフック追加に
//! binary 変更・再コンパイルは要らない（configs と疎結合・スケールする）。どのフックが macOS 専用か等の
//! 知識は manifest 側（ghostty の `os = "darwin"` ＋ コマンド本体）が持ち、エンジンは関知しない。
//!
//! フックは onchange gate（[`crate::hooks::onchange`]）を通す。ユニットのソースハッシュ＋コマンド内容が
//! 前回適用時と同じならスキップ、変化（初回・**コマンド変更**を含む）なら実行する。用途は配置後の副作用
//! （bat cache 再構築・symlink 生成）。生きた外部状態への反映は hooks ではなく `output.cmd` step が担う
//! （毎 apply・冪等契約）ため、頻度軸（`frequency`）は持たない。
//!
//! プログラムが PATH に無い（未インストール）ときは skip してメッセージだけ出す（`command -v` ガード
//! 相当を、ツール名を持たずに汎用化）。実際の spawn と `argv[0]` 解決は [`mod@exec`] が担う。未インストール
//! 時はハッシュを保存しないので、後で入れたら再実行される。トップレベル `when`（ユニット gate,
//! `deps` / `os`）が false のユニットは配置ごと skip されるため hooks も走らない（＝ `when.os` でフックを
//! 分岐できる）。

mod exec;
pub(crate) mod onchange;

use crate::manifest::Hook;
use exec::{Exec, exec};
use onchange::State;
use std::path::Path;

/// ユニットが宣言した全 hooks を宣言順に onchange gate を通して実行する（[`crate::apply`] が配置後に
/// 呼ぶ）。ソースハッシュは 1 回だけ計算して全フックで使い回す。
pub fn run_unit_hooks(
    unit_dir: &Path,
    unit_rel: &str,
    hooks: &[Hook],
    state: &mut State,
) -> Result<(), String> {
    if hooks.is_empty() {
        return Ok(());
    }
    let source_hash = onchange::hash_dir(unit_dir)?;
    for hook in hooks {
        run_onchange(&hook.cmd, unit_dir, unit_rel, &source_hash, state)?;
    }
    Ok(())
}

/// 1 つのフック（hooks は onchange 固定）を onchange gate を通して実行する。
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
