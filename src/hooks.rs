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
use std::path::{Path, PathBuf};
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
        run_one(argv, unit_dir, unit_rel, &source_hash, state)?;
    }
    Ok(())
}

/// ユニットの全 hooks を **onchange gate を通さず無条件に**実行する（[`crate::color`] が
/// テーマ切替後の reload に使う）。
///
/// `dotfiles color` はテーマ**状態**だけを変えユニットのソースは変えないため source ハッシュは
/// 不変で、通常の onchange gate（[`run_unit_hooks`]）では reload hook が「ソース不変」で skip される。
/// そこで `theme = "source"` のユニットに限り、その hooks（reload など）をここで強制発火させる。
/// 状態（`hooks.toml`）には依存も記録もしない ― force は「冪等な onchange hook を明示的に再走させる」
/// だけで、再走しても安全であることは onchange hook の冪等性契約（§13）が担保する。実体化の解決
/// （bare 名の未インストール skip / 区切り付きパスの不在エラー）は通常実行と同じ [`exec`] を共有する。
pub fn run_unit_hooks_forced(
    unit_dir: &Path,
    unit_rel: &str,
    hooks: &[Vec<String>],
) -> Result<(), String> {
    for argv in hooks {
        let Some(program) = argv.first() else {
            // manifest 検証で非空を保証済みだが防御的に弾く（[`run_one`] と同方針）。
            return Err(format!("{unit_rel}: hook コマンドが空です"));
        };
        let label = argv.join(" ");
        match exec(argv, unit_dir)? {
            Exec::Ran => println!("hook: {label} ({unit_rel}) → ran (forced)"),
            Exec::ProgramMissing => {
                println!("hook: {label} ({unit_rel}) → skip ({program} が PATH にない)");
            }
        }
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

/// フック実行の結果の区別。`ProgramMissing` は未インストール（skip 相当）、`Ran` は実行成功。
enum Exec {
    Ran,
    ProgramMissing,
}

/// argv を実行する。spawn が `NotFound` のとき、`argv[0]` が **bare コマンド名**なら未インストール
/// とみなし [`Exec::ProgramMissing`]（skip）、**絶対パス／区切り付き相対パス**なら同梱物の不在として
/// エラー（apply を止める, §13.1）。非ゼロ終了も stderr 付きでエラー、正常終了は [`Exec::Ran`]。
///
/// stdout/stderr は捨て、失敗時のみ stderr を添える（フックの進捗ノイズを apply 出力に混ぜない）。
/// 「bare 名の未インストールは skip・実行して失敗はエラー」の区別が、chezmoi の `if command -v …`
/// ガードをツール名を持たずに汎用再現する。なお `NotFound` 判定は `argv[0]` のみが対象で、
/// `["sh", "-c", …]` の内側コマンドは含まれない（内側依存は `when.deps` で gate する, §13.1）。
///
/// `current_dir` をユニットの `manifest.toml` ディレクトリ（`unit_dir`）に固定する（設計書 §13.3）。
/// これにより相対パス引数とフック自身の実行時 CWD がそのディレクトリ基準になる。プログラムパス
/// （`argv[0]`）が区切り付きの相対パス（`./script.sh` 等）のときは [`program_path`] で同ディレクトリ
/// 基準へ明示解決する ― `current_dir` に頼った相対プログラムパスの解決はプラットフォーム依存
/// （unstable; std のドキュメント参照）なため、ここで曖昧さを消す。
///
/// 先に `unit_dir` を**絶対パス化**してから program/`current_dir` の双方に使う。相対のまま
/// program を join すると `current_dir`（chdir）と二重適用され（chdir 後の CWD から更に `unit_dir`
/// を辿ってしまう）解決が壊れるため。絶対化は [`std::path::absolute`]（symlink は辿らず字句的に
/// プロセス CWD を前置するだけ）で行い、manifest dir の見た目を保つ（失敗時は握りつぶさず伝播）。
/// フックスクリプトは manifest と同じ `configs/<unit>/` に置く想定。PATH 解決される bare コマンド名
/// （`bat` 等）・絶対パスはこの基準の影響を受けない（PATH 探索・絶対参照は CWD に依らないため素通しする）。
fn exec(argv: &[String], unit_dir: &Path) -> Result<Exec, String> {
    // unit_dir を絶対化してから program/current_dir の双方に使う（二重適用回避・§13.3）。絶対化が
    // 失敗するのは getcwd 不能等のみで、その状況は CWD 相対の `configs` ソース自体が解決不能＝apply
    // 不成立なので、握りつぶさずエラーを伝播する。
    let dir = std::path::absolute(unit_dir)
        .map_err(|e| format!("{}: hook 実行ディレクトリの絶対パス化に失敗: {e}", argv[0]))?;
    let program = program_path(&argv[0], &dir);
    let output = match Command::new(&program)
        .args(&argv[1..])
        .current_dir(&dir)
        .output()
    {
        Ok(o) => o,
        // bare コマンド名（PATH 探索）の `NotFound` だけ「未インストール → skip」。絶対パス／区切り付き
        // 相対パスの `NotFound` はユニット同梱物（`configs/<unit>/…`）の不在＝typo / コミット漏れなので
        // エラーで止める（空 argv を load 時に弾くのと同じ「実体化できない typo を黙殺しない」方針, §13.1）。
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && is_bare_command(&argv[0]) => {
            return Ok(Exec::ProgramMissing);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "hook プログラムが見つかりません: {} （解決先: {}）",
                argv[0],
                program.display()
            ));
        }
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

/// `argv[0]` が PATH 探索される **bare コマンド名**か（区切りを含まない＝絶対でなくコンポーネント
/// 1 個）。`bat` は true、`./x`（`CurDir`＋`Normal`＝2）/ `dir/x`（`Normal`×2）/ `../x`
/// （`ParentDir`＋`Normal`）/ 絶対パスは false。`NotFound` 時の扱い（bare＝skip / それ以外＝エラー）と
/// [`program_path`] の解決基準（区切り付き相対だけ join）が、ともにこの区別を使う。
fn is_bare_command(arg0: &str) -> bool {
    let p = Path::new(arg0);
    !p.is_absolute() && p.components().count() <= 1
}

/// hook プログラム（`argv[0]`）の実行パスを決める（§13.3）。**区切りを含む相対パス**（`./script.sh`・
/// `dir/script.sh`・`../x` ＝ 絶対でも bare でもないもの）だけを `unit_dir`（呼び出し側で絶対化済み）に
/// join して解決する。**bare コマンド名**（`bat` 等）は PATH 探索に委ねるため、**絶対パス**はそのまま
/// 使うため、いずれも素通しする。
fn program_path(arg0: &str, unit_dir: &Path) -> PathBuf {
    let p = Path::new(arg0);
    if !p.is_absolute() && !is_bare_command(arg0) {
        unit_dir.join(p)
    } else {
        p.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIT: &str = "/u/configs/demo";

    /// 区切りを含む相対パスは unit_dir 基準へ解決する（`./` / サブディレクトリ / `..`）。
    #[test]
    fn relative_with_separator_resolves_against_unit_dir() {
        let unit = Path::new(UNIT);
        assert_eq!(program_path("./hook.sh", unit), unit.join("./hook.sh"));
        assert_eq!(program_path("bin/hook.sh", unit), unit.join("bin/hook.sh"));
        assert_eq!(
            program_path("../shared/hook.sh", unit),
            unit.join("../shared/hook.sh")
        );
    }

    /// 区切りを含まない bare 名は PATH 探索に委ねる（unit_dir を前置しない）。
    #[test]
    fn bare_command_name_is_left_for_path_lookup() {
        let unit = Path::new(UNIT);
        assert_eq!(program_path("bat", unit), PathBuf::from("bat"));
        assert_eq!(program_path("faketool", unit), PathBuf::from("faketool"));
    }

    /// `is_bare_command`: bare 名のみ true。区切り付き相対・絶対は false（＝NotFound でエラー側）。
    #[test]
    fn is_bare_command_only_for_separatorless_names() {
        assert!(is_bare_command("bat"));
        assert!(is_bare_command("faketool"));
        assert!(!is_bare_command("./hook.sh"));
        assert!(!is_bare_command("bin/hook.sh"));
        assert!(!is_bare_command("../shared/hook.sh"));
        assert!(!is_bare_command("/usr/bin/bat"));
    }

    /// 絶対パスはそのまま（CWD 非依存）。
    #[test]
    fn absolute_path_is_untouched() {
        let unit = Path::new(UNIT);
        assert_eq!(
            program_path("/usr/bin/bat", unit),
            PathBuf::from("/usr/bin/bat")
        );
    }
}
