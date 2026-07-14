//! `gh repo clone` してから `ghq migrate` でリポジトリを移設し、移設先パスを
//! stdout に出力する。
//!
//! 親シェルの cd は別プロセスから行えないため、移設先パスを stdout に出し、
//! 薄い fish shim（gh-clone.fish）が `cd` する（shim 定石）。stdout はパスのみ、
//! それ以外（gh/ghq の進捗）は stderr に出す。

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use clap::Parser;

/// gh repo clone してから ghq migrate し、移設先パスを stdout に出力する。
#[derive(Parser)]
#[command(
    name = "gh-clone",
    version,
    about = "Clone a repo with gh then ghq migrate, printing the migrated path"
)]
struct Cli {
    /// clone するリポジトリ（owner/repo）。
    repo: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if !command_exists("gh") {
        eprintln!("gh-clone: gh command not found.");
        return ExitCode::from(127);
    }
    if !command_exists("ghq") {
        eprintln!("gh-clone: ghq command not found.");
        return ExitCode::from(127);
    }

    let repo_spec = &cli.repo;

    // gh repo clone: 進捗は stderr 継承。gh の stdout は捕捉して stderr に流し、
    // 本プロセスの stdout（= 移設先パス）を汚さない。
    let clone = match Command::new("gh")
        .args(["repo", "clone", repo_spec])
        .stderr(Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(_) => {
            eprintln!("gh-clone: failed to run gh");
            return ExitCode::FAILURE;
        }
    };
    let _ = io::stderr().write_all(&clone.stdout);
    if !clone.status.success() {
        return ExitCode::from(clone.status.code().unwrap_or(1) as u8);
    }

    // path basename 相当。
    let repo_dir = Path::new(repo_spec)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| repo_spec.clone());

    let migrate = match Command::new("ghq")
        .args(["migrate", "-y", &repo_dir])
        .stderr(Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(_) => {
            eprintln!("gh-clone: failed to run ghq");
            return ExitCode::FAILURE;
        }
    };
    if !migrate.status.success() {
        return ExitCode::from(migrate.status.code().unwrap_or(1) as u8);
    }

    let migrated_path = String::from_utf8_lossy(&migrate.stdout).trim().to_string();
    println!("{migrated_path}");
    ExitCode::SUCCESS
}

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}
