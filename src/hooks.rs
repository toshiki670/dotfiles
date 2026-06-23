//! onchange フック（§13, S5）: ユニット配置後（after フェーズ）に、manifest が宣言した
//! **コマンド（argv）**を onchange gate を通して実行する**汎用エンジン**。
//!
//! ツール固有のロジックは binary に一切持たない。フックは manifest の `hooks` 属性が
//! argv（コマンド列）を**データ**として宣言し、本モジュールはそれを実行するだけ ―
//! [`crate::generate`] の `cmd`（manifest のコマンドをデータとして実行）と同じ思想で、
//! 新ツールのフック追加に binary 変更・再コンパイルは要らない（configs と疎結合・スケールする）。
//! どのフックが macOS 専用か等の知識は manifest 側（ghostty の `os = "darwin"` ＋ コマンド本体）が
//! 持ち、エンジンは関知しない。
//!
//! 実行は onchange gate（[`crate::onchange`]）を通す: ユニットのソースハッシュ＋コマンド内容が
//! 前回適用時と同じならスキップ、変化（初回・**コマンド変更**を含む）なら実行する。プログラムが
//! PATH に無い（未インストール）ときは skip してハッシュを保存せず、後で入れたら再実行されるように
//! する（chezmoi の `command -v` ガード相当を、ツール名を持たずに汎用化）。トップレベル `when`
//! （ユニット gate, `deps` / `os`）が false のユニットは配置ごと skip されるため hooks も走らない
//! （＝ `when.os` でフックを分岐できる）。

use crate::onchange::{self, State};
use std::path::Path;
use std::process::Command;

/// ユニットが宣言した全 hooks を onchange gate を通して順に実行する（[`crate::apply`] が配置後に呼ぶ）。
/// `hooks` は argv（コマンド列）の配列。ソースハッシュは 1 回だけ計算して各フックで使い回す。
pub fn run_unit_hooks(
    unit_dir: &Path,
    unit_rel: &str,
    hooks: &[Vec<String>],
    state: &mut State,
) -> Result<(), String> {
    if hooks.is_empty() {
        return Ok(());
    }
    let source_hash = onchange::hash_dir(unit_dir)?;
    for argv in hooks {
        run_one(argv, unit_rel, &source_hash, state)?;
    }
    Ok(())
}

/// 1 フック（argv）を onchange gate を通して実行する。
///
/// 状態キーは `<unit>::<コマンドの短ハッシュ>`。コマンド内容をキーに織り込むことで、manifest 上で
/// **コマンドを変えた場合も新しいキー＝再実行**になる（`manifest.toml` はソースハッシュ対象外なので、
/// これが無いとコマンド変更を取りこぼす）。値はユニットの**ソース**ハッシュ（中身の変化で再実行）。
fn run_one(
    argv: &[String],
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

    match exec(argv)? {
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

/// フック実行の結果の区別。`ProgramMissing` は未インストール（skip 相当）、`Ran` は実行成功。
enum Exec {
    Ran,
    ProgramMissing,
}

/// argv を実行する。spawn が `NotFound`（プログラム未インストール）なら [`Exec::ProgramMissing`]、
/// 非ゼロ終了は stderr 付きでエラー（apply を止める）、正常終了は [`Exec::Ran`]。
///
/// stdout/stderr は捨て、失敗時のみ stderr を添える（フックの進捗ノイズを apply 出力に混ぜない）。
/// 「未インストールは skip・実行して失敗はエラー」の区別が、chezmoi の `if command -v …` ガードを
/// ツール名を持たずに汎用再現する。なお `NotFound` 判定は `argv[0]` のみが対象で、`["sh", "-c", …]`
/// の内側コマンドは含まれない（内側依存は `when.deps` で gate する, §13.1）。
///
/// `current_dir` は未設定 ＝ プロセス CWD を継承する。設計書 §13.3 の確定仕様（相対パス hook は
/// ユニットの `manifest.toml` ディレクトリ基準）とは差分があり、追従は #498。現状の hooks は絶対
/// パス / `$HOME` / PATH 解決で CWD 非依存なので顕在化しない。
fn exec(argv: &[String]) -> Result<Exec, String> {
    let output = match Command::new(&argv[0]).args(&argv[1..]).output() {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Exec::ProgramMissing),
        Err(e) => return Err(format!("{}: 実行失敗: {e}", argv[0])),
    };
    if output.status.success() {
        Ok(Exec::Ran)
    } else {
        Err(format!(
            "hook {argv:?} が異常終了 ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
