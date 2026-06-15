//! nvim プラグインを同期（Lazy sync）し、lazy-lock.json を chezmoi ソースへ
//! re-add する。旧 `v-sync.fish` の移植。

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    if !command_exists("nvim") {
        eprintln!("v-sync: nvim command not found.");
        return ExitCode::from(127);
    }
    if !command_exists("chezmoi") {
        eprintln!("v-sync: chezmoi command not found.");
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

    println!("v-sync: re-adding lazy-lock.json into chezmoi source...");
    let home = std::env::var("HOME").unwrap_or_default();
    let lock = format!("{home}/.config/nvim/lazy-lock.json");
    match Command::new("chezmoi").arg("re-add").arg(lock).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::FAILURE,
    }
}

/// `command -q` 相当: PATH 上に実行ファイルがあるか。
fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}
