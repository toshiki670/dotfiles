//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各コマンド（`crates/*`）に委譲しつつ、それらを取り込む器。
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` / `list` サブコマンドは dotfiles ネイティブ化（Epic #453）の一部：
//! 固定ソース `configs/` を走査し、`manifest.toml` に従って copy 層で配置する（S1 / #455）。
//! `apply` は配置（ディレクトリ単位 copy・再帰・パーミッション）、`list` は分散 manifest を
//! 集約した配置先一覧を担う。

use clap::{Parser, Subcommand};
use std::path::Path;

mod apply;
mod discover;
mod list;
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
    /// 固定ソース `configs/` を走査し設定を配置する（copy 層）。
    Apply,
    /// configs の manifest を集約し、配置先一覧を表示する。
    List,
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
        Some(Commands::List) => {
            if let Err(e) = list::run(Path::new("configs")) {
                eprintln!("dotfiles list: {e}");
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
