//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各衛星コマンド（`crates/*`）に委譲しており、本体は将来
//! それらを取り込む器（all-in-one 化の予定地）。当面は clap で `--version` /
//! `--help` を提供し、バージョンの source of truth（タグ `v{version}`）を担う。

use clap::Parser;

/// toshiki670/dotfiles 本体（core）。
#[derive(Parser)]
#[command(
    name = "dotfiles",
    version,
    about = "toshiki670/dotfiles 本体（core）コマンド",
    long_about = "toshiki670/dotfiles 本体（core）。現状は chezmoi と各衛星コマンドに委譲しており、\n将来それらを取り込む器。当面はバージョン確認用。"
)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
    // 現状サブコマンドは持たない。`--version` / `--help` は clap が処理する。
    println!("dotfiles {}", env!("CARGO_PKG_VERSION"));
}
