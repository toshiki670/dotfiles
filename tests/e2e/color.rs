//! `dotfiles color`（S6 sample/#460 ＋ S7 切替/#461）の E2E。
//!
//! - `color sample`: 旧 `crates/color` から吸収した ANSI 確認表が、見出し（16/256 Colors）と
//!   期待の ANSI エスケープシーケンス（リセット・16 色の前景/背景・256 色の背景）を出力する。
//! - `color dark|light|auto`（S7）: 状態ファイル（`~/.config/dotfiles/theme`）への書き込み・
//!   `when.theme` overlay による fragment 選択・`theme = "source"` ユニットの reload 強制発火を、
//!   **架空ユニット（app）＋ PATH スタブ reload**で hermetic に検証する（実 ghostty/pkill に依存しない, §15.2）。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use rstest::rstest;
use std::fs;
use std::path::Path;

#[test]
fn sample_help_succeeds() {
    dotfiles()
        .args(["color", "sample", "--help"])
        .assert()
        .success();
}

#[test]
fn sample_prints_16_and_256_color_tables() {
    dotfiles()
        .args(["color", "sample"])
        .assert()
        .success()
        .stdout(predicate::str::contains("16 Colors"))
        .stdout(predicate::str::contains("256 Colors"));
}

#[test]
fn sample_emits_expected_ansi_sequences() {
    dotfiles()
        .args(["color", "sample"])
        .assert()
        .success()
        // リセット（全シーケンス共通の終端）。
        .stdout(predicate::str::contains("\x1b[0m"))
        // 16 色: 白背景 + 明るい白前景の組み合わせ。
        .stdout(predicate::str::contains("\x1b[47m\x1b[1;37m"))
        // 256 色: 背景 5;NNN + 前景 5;0 の組み合わせ（最初のセル）。
        .stdout(predicate::str::contains("\x1b[48;5;0m\x1b[38;5;0m"));
}

/// `theme = "source"` の架空ユニット（base ＋ when.theme overlay ＋ reload hook）を書き出す。
/// reload hook は PATH スタブ `fake-reload`（HOME 配下に marker を touch）で、実ツール（ghostty/pkill）に
/// 依存しない ― エンジンがツール名を持たず argv を実行するだけであることを使う（§15.2 hermetic）。
fn write_theme_source_unit(work: &Path) {
    let unit = work.join("configs/app");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/app/out.conf\"\n\
         strategy = \"concat\"\n\
         theme = \"source\"\n\
         hooks = [[\"fake-reload\"]]\n\
         [[overlay]]\n\
         src = \"config\"\n\
         [[overlay]]\n\
         src = \"theme-auto\"\n\
         when = { theme = \"auto\" }\n\
         [[overlay]]\n\
         src = \"theme-dark\"\n\
         when = { theme = \"dark\" }\n\
         [[overlay]]\n\
         src = \"theme-light\"\n\
         when = { theme = \"light\" }\n",
    )
    .unwrap();
    fs::write(unit.join("config"), "BASE\n").unwrap();
    fs::write(unit.join("theme-auto"), "MODE=auto\n").unwrap();
    fs::write(unit.join("theme-dark"), "MODE=dark\n").unwrap();
    fs::write(unit.join("theme-light"), "MODE=light\n").unwrap();
}

/// `color <mode>` が状態ファイルへ書き、`when.theme` overlay が該当 fragment を 1 つだけ選んで合成し、
/// `theme = "source"` の reload hook を発火する。
#[cfg(unix)]
#[rstest]
#[case("dark", "MODE=dark")]
#[case("light", "MODE=light")]
#[case("auto", "MODE=auto")]
fn color_set_writes_state_composes_theme_and_reloads(#[case] mode: &str, #[case] expect: &str) {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    // marker はシェルのリダイレクトだけで作る（PATH を stub dir に絞るため touch 等の外部コマンドに頼らない）。
    write_stub(bin.path(), "fake-reload", ": > \"$HOME/reloaded-marker\"\n");
    write_theme_source_unit(work.path());

    dotfiles()
        .args(["color", mode])
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    // ① 状態ファイルにモードが書かれる。
    assert_eq!(
        fs::read_to_string(home.path().join(".config/dotfiles/theme"))
            .unwrap()
            .trim(),
        mode,
    );

    // ② dst が base ＋ 該当テーマ断片で合成され、他テーマ断片は混入しない。
    let out = fs::read_to_string(home.path().join(".config/app/out.conf")).unwrap();
    assert!(out.contains("BASE"), "base が無い: {out}");
    assert!(out.contains(expect), "{mode} の fragment が無い: {out}");
    for other in ["MODE=auto", "MODE=dark", "MODE=light"] {
        if other != expect {
            assert!(
                !out.contains(other),
                "{mode} に他テーマ {other} が混入: {out}"
            );
        }
    }

    // ③ theme=source の reload hook が発火する。
    assert!(
        home.path().join("reloaded-marker").exists(),
        "reload hook が走っていない",
    );
}

/// テーマ**状態**だけ変えた（fixture ソースは不変）時も、出力が再合成され reload が**強制**発火する。
/// apply の onchange gate はソースハッシュで skip するが、color は theme=source の hooks を強制実行する
/// ― これが「ソースは変わらないのにテーマだけ切り替える」S7 の核心経路（§10.2.1）。
#[cfg(unix)]
#[test]
fn color_set_recomposes_and_force_reloads_on_state_change_only() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    // marker はシェルのリダイレクトだけで作る（PATH を stub dir に絞るため touch 等の外部コマンドに頼らない）。
    write_stub(bin.path(), "fake-reload", ": > \"$HOME/reloaded-marker\"\n");
    write_theme_source_unit(work.path());

    let run = |mode: &str| {
        dotfiles()
            .args(["color", mode])
            .current_dir(work.path())
            .env("HOME", home.path())
            .env("PATH", bin.path())
            .assert()
            .success();
    };

    // 1 回目（auto）で onchange ハッシュを確定させる（fake-reload の前回ハッシュを保存）。
    run("auto");
    let marker = home.path().join("reloaded-marker");
    fs::remove_file(&marker).unwrap();

    // 2 回目（dark）: fixture ソースは不変なので apply の onchange は fake-reload を skip するが、
    // color が theme=source の hooks を強制発火するので marker が再生成される。
    run("dark");
    assert!(
        marker.exists(),
        "ソース不変でも color は reload を強制発火するはず",
    );

    // 出力もソースハッシュ不変のまま dark へ再合成される（when.theme の選択は状態に追従）。
    let out = fs::read_to_string(home.path().join(".config/app/out.conf")).unwrap();
    assert!(
        out.contains("MODE=dark") && !out.contains("MODE=auto"),
        "状態変化で再合成されていない: {out}",
    );
}
