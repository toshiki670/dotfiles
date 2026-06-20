//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各コマンド（`crates/*`）に委譲しつつ、それらを取り込む器。
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` / `list` サブコマンドは dotfiles ネイティブ化（Epic #453）の一部：
//! 固定ソース `configs/` を走査し、`manifest.toml` に従って配置する。配置は **2軸**
//! （生成方式 `kind`=copy/generate × 合成 `strategy`=concat/json-shallow）＋条件付き overlay
//! （`when` gate）で捉える（設計書 §5 / §5.5）。copy はツリー配置、generate / overlay 明示は
//! ファイル合成（[`compose`]）を通り、`deps` / `os` はユニット単位 gate（[`gate`]）。
//! `apply` は配置、`list` は分散 manifest を集約した配置先一覧を担う。

use clap::{Parser, Subcommand};
use std::path::Path;

mod apply;
mod compose;
mod copy;
mod discover;
mod gate;
mod generate;
mod list;
mod manifest;
mod strategy;

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
