//! フックプログラムの実行プリミティブ: argv を spawn し、`argv[0]` を
//! 「PATH 探索の bare コマンド名 / `unit_dir` 基準の区切り付き相対パス / 絶対パス」で解決する。
//!
//! 未インストール（bare 名の `NotFound`）は [`Exec::ProgramMissing`]（skip 相当）、ユニット同梱物の
//! 不在（区切り付き相対・絶対の `NotFound`）はエラーとして区別する ― `command -v` ガードを
//! ツール名を持たずに汎用再現する部分。頻度による実行モデルの分岐（onchange gate / 無条件実行）は
//! 上位の [`crate::hooks`] が担い、本モジュールは「どう起動し、`argv[0]` をどう解決するか」だけを持つ。

use std::path::{Path, PathBuf};
use std::process::Command;

/// フック実行の結果の区別。`ProgramMissing` は未インストール（skip 相当）、`Ran` は実行成功。
pub enum Exec {
    Ran,
    ProgramMissing,
}

/// argv を実行する。spawn が `NotFound` のとき、`argv[0]` が **bare コマンド名**なら未インストール
/// とみなし [`Exec::ProgramMissing`]（skip）、**絶対パス／区切り付き相対パス**なら同梱物の不在として
/// エラー（apply を止める）。非ゼロ終了も stderr 付きでエラー、正常終了は [`Exec::Ran`]。
///
/// stdout/stderr は捨て、失敗時のみ stderr を添える（フックの進捗ノイズを apply 出力に混ぜない）。
/// 「bare 名の未インストールは skip・実行して失敗はエラー」の区別が、`if command -v …`
/// ガードをツール名を持たずに汎用再現する。なお `NotFound` 判定は `argv[0]` のみが対象で、
/// `["sh", "-c", …]` の内側コマンドは含まれない（内側依存は `when.deps` で gate する）。
///
/// `current_dir` をユニットの `manifest.toml` ディレクトリ（`unit_dir`）に固定する。
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
/// （`faketool` 等）・絶対パスはこの基準の影響を受けない（PATH 探索・絶対参照は CWD に依らないため素通しする）。
pub fn exec(argv: &[String], unit_dir: &Path) -> Result<Exec, String> {
    // unit_dir を絶対化してから program/current_dir の双方に使う（二重適用回避）。絶対化が
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
        // エラーで止める（空 argv を load 時に弾くのと同じ「実体化できない typo を黙殺しない」方針）。
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
/// 1 個）。`faketool` は true、`./x`（`CurDir`＋`Normal`＝2）/ `dir/x`（`Normal`×2）/ `../x`
/// （`ParentDir`＋`Normal`）/ 絶対パスは false。`NotFound` 時の扱い（bare＝skip / それ以外＝エラー）と
/// [`program_path`] の解決基準（区切り付き相対だけ join）が、ともにこの区別を使う。
fn is_bare_command(arg0: &str) -> bool {
    let p = Path::new(arg0);
    !p.is_absolute() && p.components().count() <= 1
}

/// hook プログラム（`argv[0]`）の実行パスを決める。**区切りを含む相対パス**（`./script.sh`・
/// `dir/script.sh`・`../x` ＝ 絶対でも bare でもないもの）だけを `unit_dir`（呼び出し側で絶対化済み）に
/// join して解決する。**bare コマンド名**（`faketool` 等）は PATH 探索に委ねるため、**絶対パス**はそのまま
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
        assert_eq!(program_path("foo", unit), PathBuf::from("foo"));
        assert_eq!(program_path("faketool", unit), PathBuf::from("faketool"));
    }

    /// `is_bare_command`: bare 名のみ true。区切り付き相対・絶対は false（＝NotFound でエラー側）。
    #[test]
    fn is_bare_command_only_for_separatorless_names() {
        assert!(is_bare_command("foo"));
        assert!(is_bare_command("faketool"));
        assert!(!is_bare_command("./hook.sh"));
        assert!(!is_bare_command("bin/hook.sh"));
        assert!(!is_bare_command("../shared/hook.sh"));
        assert!(!is_bare_command("/usr/bin/foo"));
    }

    /// 絶対パスはそのまま（CWD 非依存）。
    #[test]
    fn absolute_path_is_untouched() {
        let unit = Path::new(UNIT);
        assert_eq!(
            program_path("/usr/bin/foo", unit),
            PathBuf::from("/usr/bin/foo")
        );
    }
}
