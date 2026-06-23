//! `dotfiles color sample`（S6/#460）の E2E。
//!
//! 旧 `crates/color` から吸収した ANSI 確認表が、見出し（16/256 Colors）と
//! 期待の ANSI エスケープシーケンス（リセット・16 色の前景/背景・256 色の背景）を
//! 出力することを検証する（受け入れ条件: 出力に期待の ANSI シーケンスが含まれる）。

use crate::dotfiles;
use predicates::prelude::*;

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
