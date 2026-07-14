//! `upgrade`: brew / mise / cargo を一括更新する。
//!
//! PATH 上に存在するパッケージマネージャだけを順に更新する。各ステップは失敗しても
//! 続行する（[`super::command::run`]）。引数は取らないが clap で `--help` /
//! `--version` を提供する。

use super::banner::header;
use super::command::{self, command_exists};

pub fn run() {
    header("Upgrading Environment");

    if command_exists("brew") {
        command::run("Homebrew", "brew", &["upgrade"]);
    }

    if command_exists("mise") {
        command::run("mise upgrade", "mise", &["upgrade"]);
        command::run("mise reshim", "mise", &["reshim"]);
    }

    if command_exists("cargo") && command_exists("cargo-install-update") {
        command::run("Cargo", "cargo", &["install-update", "--locked", "--all"]);
    }

    header("Done");
}
