//! `dotfiles`（core）の CLI 定義とサブコマンドのディスパッチ。
//!
//! [`run`] が `src/bin/dotfiles.rs` の数行シムから呼ばれる入口。各サブコマンドの実体は
//! core 配下の対応モジュール（[`super::apply`] / [`super::list`] / [`super::secret`] …）にある。

use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{apply, color, doctor, list, profile, secret, source};

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

/// `dotfiles` の入口。引数を解釈し、サブコマンドへディスパッチする。
pub fn run() {
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
