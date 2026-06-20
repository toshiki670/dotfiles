//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各コマンド（`crates/*`）に委譲しつつ、それらを取り込む器。
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` サブコマンドは dotfiles ネイティブ化（Epic #453）の最小骨格（S0 / #454）：
//! 固定ソース `configs/` を走査し、`manifest.toml`（dst / kind=copy）に従って配置する。

use clap::{Parser, Subcommand};
use std::path::Path;

mod apply;
mod manifest;

/// toshiki670/dotfiles 本体（core）。
#[derive(Parser)]
#[command(
    name = "dotfiles",
    version,
    about = "toshiki670/dotfiles 本体（core）コマンド",
    long_about = "toshiki670/dotfiles 本体（core）。設定の管理・配置を担う。\nサブコマンドを指定しない場合はバージョンを表示する。"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 固定ソース `configs/` を走査し設定を配置する（S0: copy のみ）。
    Apply,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Apply) => {
            if let Err(e) = run_apply() {
                eprintln!("dotfiles apply: {e}");
                std::process::exit(1);
            }
        }
        // サブコマンドなし: 従来どおりバージョンを表示する。
        None => println!("dotfiles {}", env!("CARGO_PKG_VERSION")),
    }
}

/// `dotfiles apply`：CWD 相対の固定ソース `configs/` を、HOME を基点に配置する。
fn run_apply() -> Result<(), String> {
    let home = std::env::var_os("HOME").ok_or_else(|| "HOME が未設定".to_string())?;
    apply::run(Path::new("configs"), Path::new(&home))
}
