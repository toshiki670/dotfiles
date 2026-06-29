//! upstream リモートを追従するマージヘルパー。
//!
//! 旧 `bin/git-upstream`（Python）の移植。`git upstream` サブコマンドとして、
//! `upstream/master` を fetch してマージする。`-i <remote_url>` で upstream
//! リモートを初期化する。
//!
//! 参考: <https://qiita.com/xtetsuji/items/555a1ef19ed21ee42873>

use std::process::{Command, ExitCode};

use clap::Parser;

const CMDNAME: &str = "git-upstream";
const REPOSITORY: &str = "upstream";
const UPSTREAM_PATH: &str = "upstream/master";

/// Upstream merger.
#[derive(Parser)]
#[command(
    name = CMDNAME,
    version,
    about = "Upstream merger.",
    after_help = "Example:\n  $ git-upstream\n  $ git-upstream -i https://github.com/toshiki670/dotfiles.git"
)]
struct Cli {
    /// Initialize: upstream リモートを <remote_url> で追加する。
    #[arg(short = 'i', value_name = "remote_url")]
    init: Option<String>,
}

pub fn run() -> ExitCode {
    // root での実行は非対応。
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("{CMDNAME}: Running this script as root is not supported.");
        return ExitCode::from(4);
    }

    // git コマンドの存在確認。
    if !git_available() {
        eprintln!("{CMDNAME}: Git command not found.");
        return ExitCode::from(8);
    }

    let cli = Cli::parse();
    let code = match cli.init {
        Some(remote_url) => initialize(&remote_url),
        None => merge(),
    };
    ExitCode::from(code as u8)
}

/// `git --version` が成功するかで git の存在を確認する。
fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// upstream リモートが登録済みか判定する。
fn has_upstream() -> bool {
    match Command::new("git").arg("remote").output() {
        Ok(out) => remote_exists(&String::from_utf8_lossy(&out.stdout), REPOSITORY),
        Err(_) => false,
    }
}

/// `git remote` の出力（1 行 1 リモート名）に指定したリモートが含まれるか判定する。
fn remote_exists(remotes_output: &str, name: &str) -> bool {
    remotes_output.lines().any(|line| line.trim() == name)
}

/// `git <args>` を stdio 継承で実行し、終了コードを返す。
fn git(args: &[&str]) -> i32 {
    match Command::new("git").args(args).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => 1,
    }
}

/// upstream を fetch して `upstream/master` をマージする。
fn merge() -> i32 {
    if !has_upstream() {
        println!("{CMDNAME}: Git hasn't `{REPOSITORY}' REPOSITORY.");
        println!("{CMDNAME}: Please command below:");
        println!("{CMDNAME}: $ {CMDNAME} -i <remote_url>");
        return 32;
    }

    let rc = git(&["fetch", REPOSITORY]);
    if rc != 0 {
        return rc;
    }

    git(&["merge", UPSTREAM_PATH])
}

/// upstream リモートを追加して fetch する。
fn initialize(remote_url: &str) -> i32 {
    if has_upstream() {
        eprintln!("{CMDNAME}: Already initialized.");
        return 64;
    }

    let rc = git(&["remote", "add", REPOSITORY, remote_url]);
    if rc != 0 {
        return rc;
    }

    git(&["fetch", REPOSITORY])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_exists_finds_named_remote() {
        let output = "origin\nupstream\n";
        assert!(remote_exists(output, "upstream"));
        assert!(remote_exists(output, "origin"));
    }

    #[test]
    fn remote_exists_returns_false_when_absent() {
        assert!(!remote_exists("origin\n", "upstream"));
    }

    #[test]
    fn remote_exists_ignores_surrounding_whitespace() {
        assert!(remote_exists("  upstream  \n", "upstream"));
    }

    #[test]
    fn remote_exists_is_exact_match() {
        assert!(!remote_exists("upstream-mirror\n", "upstream"));
    }
}
