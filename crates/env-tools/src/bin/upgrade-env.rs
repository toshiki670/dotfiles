//! `upgrade-env`: brew / mise / cargo を一括更新する（旧 `bin/upgrade-env` の移植）。
//!
//! PATH 上に存在するパッケージマネージャだけを順に更新する。各ステップは失敗しても
//! 続行する（[`env_tools::command::run`]）。引数は取らないが clap で `--help` /
//! `--version` を提供する。

use clap::Parser;
use env_tools::banner::header;
use env_tools::command::{command_exists, run};

/// 各パッケージマネージャとそのツールを一括更新する。
#[derive(Parser)]
#[command(
    name = "upgrade-env",
    version,
    about = "Upgrade all installed package managers and their tools"
)]
struct Cli {}

fn main() {
    // 引数は取らないが、clap で --help / --version を提供する。
    let _ = Cli::parse();

    header("Upgrading Environment");

    if command_exists("brew") {
        run("Homebrew", "brew", &["upgrade"]);
    }

    if command_exists("mise") {
        run("mise upgrade", "mise", &["upgrade"]);
        run("mise reshim", "mise", &["reshim"]);
    }

    if command_exists("cargo") && command_exists("cargo-install-update") {
        run("Cargo", "cargo", &["install-update", "--locked", "--all"]);
    }

    header("Done");
}
