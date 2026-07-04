//! nvim プラグインを同期（Lazy sync）し、lazy-lock.json を configs/nvim へ
//! 書き戻す。旧 `v-sync.fish` の移植。

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "v-sync",
    version,
    about = "Sync Neovim plugins and write lazy-lock.json back into configs/nvim"
)]
struct Cli {}

fn main() -> ExitCode {
    // 引数は取らないが、clap で --help / --version を提供する。
    let _ = Cli::parse();

    if !command_exists("nvim") {
        eprintln!("v-sync: nvim command not found.");
        return ExitCode::from(127);
    }

    println!("v-sync: syncing nvim plugins...");
    match Command::new("nvim")
        .args(["--headless", "+Lazy! sync", "+qa"])
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("v-sync: nvim plugin sync failed.");
            return ExitCode::from(status.code().unwrap_or(1) as u8);
        }
        Err(_) => {
            eprintln!("v-sync: nvim plugin sync failed.");
            return ExitCode::FAILURE;
        }
    }

    println!("v-sync: writing lazy-lock.json back into configs/nvim...");
    let home = std::env::var("HOME").unwrap_or_default();
    let live_lock = PathBuf::from(format!("{home}/.config/nvim/lazy-lock.json"));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_lock = find_repo_root(&cwd).join("configs/nvim/lazy-lock.json");

    match std::fs::copy(&live_lock, &repo_lock) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("v-sync: failed to write {}: {e}", repo_lock.display());
            ExitCode::FAILURE
        }
    }
}

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}

/// `start` から祖先を上へ辿り、`Cargo.toml` を持つ最初の dir をリポジトリルートとみなす。
fn find_repo_root(start: &Path) -> PathBuf {
    let canonical = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut cur = canonical.as_path();
    loop {
        if cur.join("Cargo.toml").is_file() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => return canonical.clone(),
        }
    }
}
