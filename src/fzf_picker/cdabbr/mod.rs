//! prompt_pwd 風の省略パス（`~` / `/` 始まり）を、各セグメントの前方一致でサブディレクトリへ
//! 再帰展開し、候補を fzf で選んで選択先を stdout に出力する。
//!
//! 親シェルの cd は別プロセスから行えないため、選択パスを stdout に出し、薄い fish
//! shim（`cdabbr.fish`）が `cd` する。fzf 不在時は候補が 1 つなら自動選択、複数なら
//! 一覧を stderr に出して失敗する。

use std::path::PathBuf;
use std::process::ExitCode;

use super::launch::{command_exists, run_fzf};
use super::parse::{AbbrBase, parse_abbr_path};
use clap::Parser;

mod expand;
use self::expand::expand_abbreviated;

/// 省略パスを展開・選択し、選択先パスを stdout に出力する。
#[derive(Parser)]
#[command(
    name = "cdabbr",
    version,
    about = "Expand a prompt_pwd-style abbreviated path and print the chosen directory"
)]
struct Cli {
    /// prompt_pwd 風の省略パス（`~` または `/` 始まり）。
    abbr_path: String,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();

    if cli.abbr_path.is_empty() {
        eprintln!("usage: cdabbr <abbreviated-path>");
        return ExitCode::FAILURE;
    }

    let Some((base_kind, segments)) = parse_abbr_path(&cli.abbr_path) else {
        eprintln!("cdabbr: path must start with ~ or /");
        return ExitCode::FAILURE;
    };

    let base = match base_kind {
        AbbrBase::Home => match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home),
            None => {
                eprintln!("cdabbr: HOME is not set");
                return ExitCode::FAILURE;
            }
        },
        AbbrBase::Root => PathBuf::from("/"),
    };

    let candidates = expand_abbreviated(&base, &segments);
    if candidates.is_empty() {
        eprintln!("cdabbr: no matching path for '{}'", cli.abbr_path);
        return ExitCode::FAILURE;
    }
    let lines: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();

    let result = match select(&lines) {
        Ok(result) => result,
        Err(code) => return code,
    };

    if let Some(path) = result {
        println!("{path}");
    }
    ExitCode::SUCCESS
}

/// 候補から 1 つ選ぶ。fzf があれば fzf（`--select-1` で 1 件なら自動選択）。無ければ
/// 1 件はそのまま、複数なら一覧を stderr に出して `Err(FAILURE)`。
fn select(lines: &[String]) -> Result<Option<String>, ExitCode> {
    if command_exists("fzf") {
        return run_fzf(
            lines,
            &[
                "--height=40%",
                "--reverse",
                "--select-1",
                "--exit-0",
                "--preview",
                "eza -1 --color=always --icons {}",
                "--preview-window",
                "right:50%:wrap",
                "--bind",
                "ctrl-/:toggle-preview",
            ],
        )
        .map_err(|_| {
            eprintln!("cdabbr: failed to run fzf");
            ExitCode::FAILURE
        });
    }

    if let [only] = lines {
        return Ok(Some(only.clone()));
    }

    eprintln!("cdabbr: multiple matches (install fzf to choose):");
    for line in lines {
        eprintln!("  {line}");
    }
    Err(ExitCode::FAILURE)
}
