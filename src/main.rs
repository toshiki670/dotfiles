//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各コマンド（`crates/*`）に委譲しつつ、それらを取り込む器。
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` / `list` サブコマンドは dotfiles ネイティブ化（Epic #453）の一部：
//! 固定ソース `configs/` を走査し、`manifest.toml` に従って配置する。配置は **2軸**
//! （生成方式 `kind`=copy/generate × 合成 `strategy`=concat/json-shallow）＋条件付き overlay
//! （`when` gate）で捉える（設計書 §5 / §5.5）。copy はツリー配置、generate / overlay 明示は
//! ファイル合成（[`compose`]）を通り、トップレベル `when`（`deps` / `os`）はユニット単位 gate（[`gate`]）。配置の直前に
//! `locals`（named value）を解決・注入する（[`resolve`] / [`inject`] / [`store`] / [`prompt`]、§9）。
//! 配置後は `hooks`（onchange フック）をユニットのソースハッシュ変化時だけ実行する
//! （[`hooks`] / [`onchange`]、§13）。`apply` は配置＋フック、`list` は配置先一覧、`secret set` は
//! named value 設定、`color sample` は ANSI 確認表（旧 `crates/color` を吸収、§10）、`doctor` は診断（雛形）。

use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::Path;

mod apply;
mod color;
mod compose;
mod copy;
mod discover;
mod doctor;
mod gate;
mod generate;
mod hooks;
mod inject;
mod list;
mod manifest;
mod onchange;
mod prompt;
mod resolve;
mod secret;
mod store;
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
    /// 固定ソース `configs/` を走査し、copy / generate / overlay 合成で設定を配置する。
    Apply,
    /// configs の manifest を集約し、配置先一覧を表示する。
    List,
    /// マシンローカル値（named value）をストアへ設定する（§9）。
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
    /// テーマ／カラー関連（§10）。現状は確認表出力の `sample` のみ。
    Color {
        #[command(subcommand)]
        action: ColorAction,
    },
    /// 依存・`locals` 未設定などを診断する（雛形・§9）。
    Doctor,
}

/// `color` のサブコマンド。テーマ手動固定（dark/light/auto）は後続スライスで追加予定（§10.2）。
#[derive(Subcommand)]
enum ColorAction {
    /// ANSI カラーコード（16 色 + 256 色）の確認表を出力する。
    Sample,
}

/// `secret` のサブコマンド。コマンド名 `secret` は仮称（非秘匿値も扱う。§16 で最終命名）。
#[derive(Subcommand)]
enum SecretAction {
    /// 名前→値をストア（`~/.config/dotfiles/local.toml`）へ設定する。
    Set { name: String, value: String },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Apply) => run_apply(),
        Some(Commands::List) => list::run(Path::new("configs")),
        Some(Commands::Secret {
            action: SecretAction::Set { name, value },
        }) => run_secret_set(&name, &value),
        Some(Commands::Color {
            action: ColorAction::Sample,
        }) => {
            color::sample();
            Ok(())
        }
        Some(Commands::Doctor) => run_doctor(),
        // サブコマンドなし: 従来どおりバージョンを表示する。
        None => {
            println!("dotfiles {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };
    if let Err(e) = result {
        eprintln!("dotfiles: {e}");
        std::process::exit(1);
    }
}

/// `dotfiles apply`：CWD 相対の固定ソース `configs/` を、HOME を基点に配置する。
fn run_apply() -> Result<(), String> {
    let home = home_dir()?;
    apply::run(Path::new("configs"), Path::new(&home))
}

/// `dotfiles secret set <name> <value>`：named value をストアへ保存する。
fn run_secret_set(name: &str, value: &str) -> Result<(), String> {
    let home = home_dir()?;
    secret::set(Path::new(&home), name, value)
}

/// `dotfiles doctor`：`configs/` の `locals` 宣言とストアを突き合わせ未設定を報告する。
fn run_doctor() -> Result<(), String> {
    let home = home_dir()?;
    doctor::run(Path::new("configs"), Path::new(&home))
}

/// HOME を取得する（dst の `~` 展開・ストアパスの基点）。
fn home_dir() -> Result<OsString, String> {
    std::env::var_os("HOME").ok_or_else(|| "HOME が未設定".to_string())
}
