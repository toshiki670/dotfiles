//! `dotfiles` 本体（core）コマンド。
//!
//! 現状は chezmoi と各コマンド（`crates/*`）に委譲しつつ、それらを取り込む器。
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` / `list` サブコマンドは dotfiles ネイティブ化（Epic #453）の一部：
//! ソース（`configs/` の実体）を二段構えで解決し（作業ツリー検出 → 埋め込み・[`source`]、§8）、
//! `manifest.toml` に従って配置する。配置は **2軸**
//! （生成方式 `kind`=copy/generate × 合成 `strategy`=concat/json-shallow）＋条件付き overlay
//! （`when` gate）で捉える（設計書 §5 / §5.5）。copy はツリー配置、generate / overlay 明示は
//! ファイル合成（[`crate::apply::compose`]）を通り、トップレベル `when`（`deps` / `os`）はユニット単位 gate（[`crate::apply::gate`]）。配置の直前に
//! `locals`（named value）を解決・注入する（解決＋注入の窓口 [`crate::locals::resolve`] / ストア [`crate::locals::store`] / 対話入力 [`crate::locals::prompt`]、§9）。
//! 配置後は `hooks`（onchange フック）をユニットのソースハッシュ変化時だけ実行する
//! （[`hooks`] / [`onchange`]、§13）。`apply` は配置＋フック、`list` は配置先一覧、`secret set` は
//! named value 設定、`profile` はマシンクラスの状態 gate 設定／表示（[`state`]、§10）、`color sample` は
//! ANSI 確認表（旧 `crates/color` を吸収、§10）、`doctor` は診断（雛形）。

#![deny(rustdoc::broken_intra_doc_links)]

use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

// 共有核（葉。多くの群が片方向で依存する契約・基盤）。
mod discover; // §6.3 走査（apply / list / doctor 共有）
mod manifest; // §6 manifest.toml スキーマ
mod source; // §8 ソース二段構え
mod state; // §10 状態駆動 gate のスカラ状態ファイル（profile / 将来の theme）

// 配置エンジン（§5 / §5.5）。子モジュール（copy / compose / generate / strategy / gate）は apply.rs が束ねる。
mod apply;

// named value（§9）。子モジュール（store / resolve / inject / prompt）は locals.rs が束ねる。
mod locals;

// onchange フック（§13。2 ファイルなので直下据え置き）。
mod hooks;
mod onchange;

// 単独ビュー / コマンド。
mod color;
mod doctor;
mod list;
mod profile;
mod secret;

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

    /// ソースルートを明示指定する（§8 の上級オプション。通常は作業ツリー検出 → 埋め込みで自動解決）。
    ///
    /// apply / list / doctor 横断で効く（`global`）。前面に出さない（`hide`）。
    #[arg(long, global = true, hide = true, value_name = "DIR")]
    source: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// ソース（作業ツリー / 埋め込み・§8）を走査し、copy / generate / overlay 合成で設定を配置する。
    Apply,
    /// configs の manifest を集約し、配置先一覧を表示する。
    List,
    /// マシンローカル値（named value）をストアへ設定する（§9）。
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
    /// マシンクラス（`profile`）の状態 gate を設定／表示する（§10）。
    ///
    /// 引数 `<name>`（例 `private`）を渡すと状態ファイルへ書き、`when = { profile = … }` の
    /// 断片が採否される。引数なしは現在の profile を表示する。未設定の既定は not-private
    /// （新規・仕事マシンへ private 設定が漏れないよう明示 opt-in）。
    Profile {
        /// 設定する profile 名（省略時は現在値を表示）。
        name: Option<String>,
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
    let source = cli.source.as_deref();
    let result = match cli.command {
        Some(Commands::Apply) => run_apply(source),
        Some(Commands::List) => run_list(source),
        Some(Commands::Secret {
            action: SecretAction::Set { name, value },
        }) => run_secret_set(&name, &value),
        Some(Commands::Profile { name }) => run_profile(name.as_deref()),
        Some(Commands::Color {
            action: ColorAction::Sample,
        }) => {
            color::sample();
            Ok(())
        }
        Some(Commands::Doctor) => run_doctor(source),
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

/// `dotfiles apply`：ソースを二段構えで解決し（§8）、HOME を基点に配置する。
/// 解決元（作業ツリー / 埋め込み / `--source`）を 1 行目に示す。
fn run_apply(source: Option<&Path>) -> Result<(), String> {
    let home = home_dir()?;
    let resolved = source::resolve(source)?;
    println!("apply: source = {}", resolved.origin());
    apply::run(resolved.root(), Path::new(&home))
}

/// `dotfiles list`：ソースを二段構えで解決し、配置先一覧を解決元の表示付きで出す。
fn run_list(source: Option<&Path>) -> Result<(), String> {
    let resolved = source::resolve(source)?;
    list::run(resolved.root(), &resolved.origin().to_string())
}

/// `dotfiles secret set <name> <value>`：named value をストアへ保存する。
fn run_secret_set(name: &str, value: &str) -> Result<(), String> {
    let home = home_dir()?;
    secret::set(Path::new(&home), name, value)
}

/// `dotfiles profile [<name>]`：`<name>` 指定で profile 状態を設定、省略で現在値を表示する。
fn run_profile(name: Option<&str>) -> Result<(), String> {
    let home = home_dir()?;
    match name {
        Some(name) => profile::set(Path::new(&home), name),
        None => profile::show(Path::new(&home)),
    }
}

/// `dotfiles doctor`：ソースの `locals` 宣言とストアを突き合わせ未設定を報告する。
fn run_doctor(source: Option<&Path>) -> Result<(), String> {
    let home = home_dir()?;
    let resolved = source::resolve(source)?;
    doctor::run(resolved.root(), Path::new(&home))
}

/// HOME を取得する（dst の `~` 展開・ストアパスの基点）。
fn home_dir() -> Result<OsString, String> {
    std::env::var_os("HOME").ok_or_else(|| "HOME が未設定".to_string())
}
