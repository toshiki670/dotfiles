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
//!
//! `secret` / `doctor` はマシンローカル値（named value, §9）: `secret set` がストア
//! （[`local_store`]）へ値を書き、`apply` が `locals` 宣言ユニットの `@@name@@` を置換注入し
//! （[`inject`]・未設定は [`prompt`] で TTY 対話）、`doctor` が未設定を診断する。

use clap::{Parser, Subcommand};
use std::path::Path;

mod apply;
mod compose;
mod copy;
mod discover;
mod doctor;
mod gate;
mod generate;
mod inject;
mod list;
mod local_store;
mod manifest;
mod prompt;
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
    /// マシンローカル値（named value）をストアへ設定する（§9）。
    ///
    /// コマンド名は仮称（§14）。`secret` は秘匿値以外（email/name 等）も扱うため、
    /// `local` 系への改名を含めて最終命名は後で確定する。
    Secret {
        #[command(subcommand)]
        action: SecretCmd,
    },
    /// マシンローカル値（`locals`）の未設定を診断する（雛形, §9）。
    Doctor,
}

#[derive(Subcommand)]
enum SecretCmd {
    /// `<name>`（例 `git.email`）に `<value>` を設定する（ストアは 0600）。
    Set { name: String, value: String },
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
        Some(Commands::Secret {
            action: SecretCmd::Set { name, value },
        }) => {
            if let Err(e) = run_secret_set(&name, &value) {
                eprintln!("dotfiles secret set: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Doctor) => {
            if let Err(e) = run_doctor() {
                eprintln!("dotfiles doctor: {e}");
                std::process::exit(1);
            }
        }
        // サブコマンドなし: 従来どおりバージョンを表示する。
        None => println!("dotfiles {}", env!("CARGO_PKG_VERSION")),
    }
}

/// HOME を解決する（apply / secret / doctor 共通）。dst の `~` 展開・ストア配置の基点。
fn home_dir() -> Result<std::path::PathBuf, String> {
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "HOME が未設定".to_string())
}

/// `dotfiles apply`：CWD 相対の固定ソース `configs/` を、HOME を基点に配置する。
fn run_apply() -> Result<(), String> {
    apply::run(Path::new("configs"), &home_dir()?)
}

/// `dotfiles secret set <name> <value>`：ストアへ値を設定する。値は決して表示しない（§9.1）。
fn run_secret_set(name: &str, value: &str) -> Result<(), String> {
    let home = home_dir()?;
    local_store::set(&home, name, value)?;
    println!(
        "dotfiles secret set: {name} を設定しました（{}）",
        local_store::path(&home).display()
    );
    Ok(())
}

/// `dotfiles doctor`：configs の `locals` 未設定を診断する。
fn run_doctor() -> Result<(), String> {
    doctor::run(Path::new("configs"), &home_dir()?)
}
