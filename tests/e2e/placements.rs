//! 期待配置集合の導出とユニット間 output 衝突検出の E2E（#593）。
//!
//! 架空 fixture（`a` / `b` 単位）で `doctor` の衝突報告を検証する: ツリーが同じディレクトリを
//! 共有しても別ファイル名なら衝突ではないこと、同一ファイルへ書けば衝突として報告されること、
//! gate（`when.deps`）が現在の環境で false でも宣言ベースで衝突を拾うことの 3 点。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// 2 つのツリーユニットを作る（`work/configs/<unit>` 配下に `file` を 1 つ持ち、`output` へ配置）。
fn write_tree_unit(work: &TempDir, unit: &str, output: &str, file: &str, when: Option<&str>) {
    let dir = work.path().join("configs").join(unit);
    fs::create_dir_all(&dir).unwrap();
    let when_line = when.map(|w| format!("when = {w}\n")).unwrap_or_default();
    fs::write(
        dir.join("manifest.toml"),
        format!("{when_line}[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"{output}\"\n"),
    )
    .unwrap();
    fs::write(dir.join(file), "x\n").unwrap();
}

/// 2 ユニットが同じディレクトリへツリー配置しても、ファイル名が違えば衝突ではない
/// （fish の conf.d / functions のような正当な合流点の再現）。
#[test]
fn doctor_reports_no_conflict_for_distinct_files_in_shared_directory() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_tree_unit(&work, "a", "~/.config/shared", "one.txt", None);
    write_tree_unit(&work, "b", "~/.config/shared", "two.txt", None);

    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("衝突はありません"));
}

/// 2 ユニットが同一パスへ output を宣言すると、doctor が衝突として報告する
/// （情報提供のみ・ブロックしない＝exit success）。
#[test]
fn doctor_reports_conflict_for_same_resolved_path() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_tree_unit(&work, "a", "~/.config/collide", "same.txt", None);
    write_tree_unit(&work, "b", "~/.config/collide", "same.txt", None);

    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("衝突が 1 件"))
        .stderr(predicate::str::contains("a"))
        .stderr(predicate::str::contains("b"));
}

/// 導出は gate 評価前の宣言ベース: `when.deps` が現在の PATH に無いバイナリを指し、実 apply では
/// skip されるユニットでも、衝突検出は環境非依存に拾う（他マシンでだけ顕在化する衝突を見逃さない）。
#[test]
fn doctor_reports_conflict_even_when_one_unit_is_gated_off_in_current_env() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_path = tempfile::tempdir().unwrap();

    write_tree_unit(
        &work,
        "a",
        "~/.config/collide",
        "same.txt",
        Some("{ deps = [\"definitely-not-a-real-binary-xyz\"] }"),
    );
    write_tree_unit(&work, "b", "~/.config/collide", "same.txt", None);

    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_path.path()) // dep gate を決定的に外す（a は実 apply では skip される）。
        .assert()
        .success()
        .stderr(predicate::str::contains("衝突が 1 件"));
}
