//! ソース二段構え（S8/#462）の解決切替を検証する E2E。
//!
//! 受け入れ条件「作業ツリー有/無での解決切替」を、`dotfiles apply` の 1 行目
//! （`apply: source = {origin}`）と実際に配置されたユニットの両面で確かめる:
//!
//! - **作業ツリー検出**: CWD のサブディレクトリから上へ辿り、ユニットを持つ `configs/` を使う
//!   （hermetic な架空 fixture `foo`）。
//! - **`--source` 明示**: 検出に掛からない CWD でも、指定したソースが最優先で使われる（fixture `bar`）。
//! - **埋め込みフォールバック**: 作業ツリーが無ければバイナリ埋め込みの出荷 configs で apply が
//!   完結する（clone 無しの自己完結。実 configs 層なので空 PATH で dep gate / bare hook を無効化）。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;

/// `<root>/configs/<unit>` に最小の copy ユニット（`dst` ＋ 1 ファイル）を作る。
fn write_unit(configs: &std::path::Path, unit: &str, dst: &str, file: &str) {
    let dir = configs.join(unit);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("manifest.toml"), format!("dst = \"{dst}\"\n")).unwrap();
    fs::write(dir.join(file), "x\n").unwrap();
}

/// 作業ツリー検出: CWD のサブディレクトリから上へ辿り、ユニットを持つ `configs/` を解決元にする。
#[test]
fn detects_working_tree_from_subdir() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_unit(
        &work.path().join("configs"),
        "foo",
        "~/.config/foo",
        "foo.conf",
    );

    // CWD をリポ相当 `work` のサブディレクトリにし、上方向検出（ancestors）が効くことを示す。
    let sub = work.path().join("sub/deep");
    fs::create_dir_all(&sub).unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(&sub)
        .env("HOME", home.path())
        .assert()
        .success()
        // 解決元が作業ツリーであることを 1 行目で示す。
        .stdout(predicate::str::contains("source = working tree"));

    // 検出した作業ツリーの fixture（埋め込みでない）が配置された証跡。
    assert!(
        home.path().join(".config/foo/foo.conf").is_file(),
        "作業ツリーの fixture が配置されていない",
    );
}

/// `--source` 明示は最優先（検出に掛からない CWD でも、指定ソースが使われる）。
#[test]
fn explicit_source_takes_precedence() {
    let explicit = tempfile::tempdir().unwrap(); // ここを `--source` のソースルートにする。
    let cwd = tempfile::tempdir().unwrap(); // configs を持たない隔離 CWD。
    let home = tempfile::tempdir().unwrap();
    write_unit(explicit.path(), "bar", "~/.config/bar", "bar.conf");

    dotfiles()
        .arg("apply")
        .arg("--source")
        .arg(explicit.path())
        .current_dir(cwd.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("source = --source"));

    assert!(
        home.path().join(".config/bar/bar.conf").is_file(),
        "--source 指定のソースが配置されていない",
    );
}

/// 埋め込みフォールバック: 作業ツリーが無ければ出荷 configs（埋め込み）で apply が完結する。
///
/// 隔離 CWD（configs 祖先なし）＋空 PATH で実行する。空 PATH は実 configs 層の決まり事
/// （[`crate::real_configs`] と同じ）で、dep gate を決定的に外し bare hook を未インストール
/// （skip）にして apply を決定的にする。
#[test]
fn falls_back_to_embedded_without_working_tree() {
    let cwd = tempfile::tempdir().unwrap(); // configs を持たない隔離 CWD（祖先にも configs 無し）。
    let home = tempfile::tempdir().unwrap();
    let empty_path = tempfile::tempdir().unwrap();

    let out = dotfiles()
        .arg("apply")
        .current_dir(cwd.path())
        .env("HOME", home.path())
        .env("PATH", empty_path.path())
        .assert()
        .success()
        // 解決元が埋め込みであることを 1 行目で示す。
        .stdout(predicate::str::contains("source = embedded"))
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();

    // 出荷 configs が埋め込みから配置された証跡: ユニット配置行（`… → … (label)`）が 1 件以上。
    // ツール名はハードコードしない（実 configs 層・data-driven）。
    let placements = stdout.lines().filter(|l| l.contains(" → ")).count();
    assert!(
        placements > 0,
        "埋め込みフォールバックでユニットが 1 つも配置されていない:\n{stdout}",
    );
}
