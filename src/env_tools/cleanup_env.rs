//! `cleanup-env`: brew / mise / cargo のキャッシュ・不要バージョンを対話的に削除する。
//!
//! 各項目は `y/N` で確認し、`-n/--dry-run` のときは実削除せず削除対象だけを表示する。
//! PATH 上に存在するパッケージマネージャだけを対象にし、各ステップは失敗しても続行する。

use super::banner::header;
use super::command::{self, command_exists};
use super::prompt::confirm;
use clap::Parser;

/// パッケージマネージャのキャッシュ・不要バージョンを対話的に削除する。
#[derive(Parser)]
#[command(
    name = "cleanup-env",
    version,
    about = "Prompt-then-cleanup for installed package managers"
)]
struct Cli {
    /// 実際には削除せず、削除対象だけを表示する。
    #[arg(short = 'n', long)]
    dry_run: bool,
}

pub fn run() {
    let cli = Cli::parse();

    header("Cleaning Up Environment");
    if cli.dry_run {
        println!("(dry-run: nothing will actually be removed)");
        println!();
    }

    if command_exists("brew") {
        if confirm("Run brew cleanup?") {
            if cli.dry_run {
                command::run(
                    "Homebrew cleanup (dry-run)",
                    "brew",
                    &["cleanup", "--dry-run"],
                );
            } else {
                command::run("Homebrew cleanup", "brew", &["cleanup"]);
            }
        }
        if confirm("Run brew autoremove?") {
            if cli.dry_run {
                command::run(
                    "Homebrew autoremove (dry-run)",
                    "brew",
                    &["autoremove", "--dry-run"],
                );
            } else {
                command::run("Homebrew autoremove", "brew", &["autoremove"]);
            }
        }
    }

    if command_exists("mise") && confirm("Run mise prune?") {
        if cli.dry_run {
            command::run("mise prune (dry-run)", "mise", &["prune", "--dry-run"]);
        } else {
            command::run("mise prune", "mise", &["prune"]);
        }
    }

    if command_exists("cargo")
        && command_exists("cargo-cache")
        && confirm("Run cargo cache --autoclean?")
    {
        if cli.dry_run {
            command::run(
                "Cargo cache (dry-run)",
                "cargo",
                &["cache", "--dry-run", "--autoclean"],
            );
        } else {
            command::run("Cargo cache", "cargo", &["cache", "--autoclean"]);
        }
    }

    header("Done");
}
