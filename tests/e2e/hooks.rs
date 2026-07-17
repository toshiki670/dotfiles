//! `dotfiles apply` のツリー末尾 output.cmd（#659）の E2E。
//!
//! ツリーユニット（`input = "."` ＋ パス output）は、末尾に `output.cmd` step を並べて配置後に
//! 走らせるコマンドを宣言できる。架空のコマンド `faketool`（PATH 先頭スタブ）と temp HOME で契約を
//! 検証する:
//! ①末尾 output.cmd は**配置のたび無条件に実行される**（ソース不変でも毎 apply 走る。onchange の
//! ような skip は無い）②トップレベル `when`（ユニット gate）が false のユニットは配置ごと skip され、
//! 末尾 output.cmd も走らない ③未インストール（PATH 不在）のコマンドは apply をエラーで止める
//! （#659 の意図的な挙動変更 ― 旧 onchange フックの「未インストール→skip」フォールバックは撤去した）
//! ④実行して非ゼロ終了は apply エラー ⑤空コマンドの load 時拒否 ⑥`list` のツリー output.cmd 表示。

use crate::{dotfiles, foreign_os, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// `faketool` の PATH スタブを置く（呼ばれるたび `$HOME/cmd-ran` へ 1 行追記＝実行回数の観測点）。
#[cfg(unix)]
fn write_faketool(bin: &Path) {
    write_stub(bin, "faketool", "printf 'x\\n' >> \"$HOME/cmd-ran\"\n");
}

/// `faketool` 実行マーカーの行数（＝実行回数）。未作成なら 0。
fn marker_lines(home: &Path) -> usize {
    fs::read_to_string(home.join("cmd-ran"))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// 末尾に `output.cmd = ["faketool"]` を持つツリーユニット（output ＋ ソースファイル）を `work` に書く。
#[cfg(unix)]
fn write_tree_cmd_unit(work: &Path) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n[[steps]]\noutput.cmd = [\"faketool\"]\n",
    )
    .unwrap();
    fs::write(unit.join("data.txt"), "v1").unwrap();
}

/// 共通の apply 実行ヘルパ（HOME と PATH を temp に固定）。
#[cfg(unix)]
fn apply(work: &Path, home: &Path, path: &Path) -> assert_cmd::assert::Assert {
    dotfiles()
        .arg("apply")
        .current_dir(work)
        .env("HOME", home)
        .env("PATH", path)
        .assert()
}

/// ①末尾 output.cmd は配置のたび無条件に実行される。ソース不変で 2 回 apply しても、毎回走るので
/// マーカーは 2 行になる（onchange のようなソース不変 skip は無い）。
#[cfg(unix)]
#[test]
fn output_cmd_runs_on_every_apply_even_when_source_unchanged() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_faketool(bin.path());
    write_tree_cmd_unit(work.path());

    // 1 回目 apply: 配置後に output.cmd が走る（マーカー 1 行）。
    apply(work.path(), home.path(), bin.path()).success();
    assert_eq!(
        marker_lines(home.path()),
        1,
        "初回の output.cmd が走っていない"
    );

    // 2 回目 apply（ソース不変）: output.cmd は毎 apply 無条件に走るのでマーカー 2 行。
    apply(work.path(), home.path(), bin.path()).success();
    assert_eq!(
        marker_lines(home.path()),
        2,
        "ツリー末尾の output.cmd はソース不変でも毎 apply 実行されるべき",
    );
}

/// ②when.os ユニット gate が false のユニットは配置ごと skip され、末尾 output.cmd も走らない。
/// `faketool` は PATH にあるが、os 不一致でユニットが skip されるためマーカーは作られない。
#[cfg(unix)]
#[test]
fn os_gate_skips_unit_including_trailing_output_cmd() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_faketool(bin.path());
    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!(
            "when = {{ os = \"{other}\" }}\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n[[steps]]\noutput.cmd = [\"faketool\"]\n",
            other = foreign_os(),
        ),
    )
    .unwrap();
    fs::write(unit.join("data.txt"), "v1").unwrap();

    apply(work.path(), home.path(), bin.path())
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("demo"));

    assert_eq!(
        marker_lines(home.path()),
        0,
        "os gate=false のユニットでは末尾 output.cmd も走らないべき",
    );
}

/// ③未インストール（PATH 不在）のコマンドは apply をエラーで止める。#659 の意図的な挙動変更 ―
/// 旧 onchange フックは bare 名の未インストールを skip したが、汎用 cmd 実行（`output.cmd`）は
/// spawn 失敗をそのままハードエラーにする（実体化できないコマンドを黙って見送らない）。
#[cfg(unix)]
#[test]
fn missing_program_fails_apply() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // faketool を置かない。

    write_tree_cmd_unit(work.path());

    apply(work.path(), home.path(), empty_bin.path())
        .failure()
        .stderr(predicate::str::contains("faketool"))
        .stderr(predicate::str::contains("実行失敗"));
}

/// ④実行して非ゼロ終了した output.cmd は apply をエラーで止める。
#[cfg(unix)]
#[test]
fn nonzero_exit_fails_apply() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 1\n");
    write_tree_cmd_unit(work.path());

    apply(work.path(), home.path(), bin.path())
        .failure()
        .stderr(predicate::str::contains("faketool"))
        .stderr(predicate::str::contains("異常終了"));
}

/// ⑤空のコマンド（argv）を持つ output.cmd は load 時に弾く（apply 失敗）。実体化できない typo を
/// 黙殺しない（step の cmd 非空検証。hooks 固有ではなく汎用の検証を通る）。
#[test]
fn empty_output_cmd_fails_at_load() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n[[steps]]\noutput.cmd = []\n",
    )
    .unwrap();
    fs::write(unit.join("f.txt"), "x\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("output.cmd"))
        .stderr(predicate::str::contains("非空"));
}

/// ⑥`dotfiles list` がツリー末尾 output.cmd を steps サマリ（`tree, output.cmd=N`）で表示する。
#[test]
fn list_shows_tree_output_cmd_attr() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n[[steps]]\noutput.cmd = [\"faketool\"]\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("tree, output.cmd=1"));
}
