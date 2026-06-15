//! dotfiles の lint オーケストレータ CLI。旧 `nix/lint.py` の移植。
//!
//! 使い方: `dotfiles-lint {fix|check} [--check-after-fix on|off] [--summary] [--json] [--verbose]`
//! 通常は `mise run lint` / `mise run check` から `cargo run` 経由で呼ばれる。

mod lint;

use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use lint::{Mode, Orchestrator, collect_files, find_repo_root};

#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    Fix,
    Check,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Toggle {
    On,
    Off,
}

#[derive(Parser)]
#[command(
    name = "dotfiles-lint",
    about = "dotfiles の lint/format オーケストレータ"
)]
struct Cli {
    /// 実行モード。
    mode: ModeArg,

    /// fix の後に check を走らせるか（既定: on）。
    #[arg(long, value_enum, default_value = "on")]
    check_after_fix: Toggle,

    /// 失敗サマリを表示する。
    #[arg(long)]
    summary: bool,

    /// 失敗を JSON で表示する。
    #[arg(long)]
    json: bool,

    /// 実行したコマンドを表示する。
    #[arg(long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let repo_root = find_repo_root(&cwd);
    let files = collect_files(&repo_root);

    let tmp = match tempfile::tempdir() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("lint: failed to create temp dir: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mode = match cli.mode {
        ModeArg::Fix => Mode::Fix,
        ModeArg::Check => Mode::Check,
    };
    let check_after_fix = cli.check_after_fix == Toggle::On;

    let mut orch = Orchestrator::new(repo_root, tmp.path().to_path_buf(), cli.verbose);
    let failed = orch.run(mode, check_after_fix, &files);

    if cli.summary {
        orch.print_summary();
    }
    if cli.json {
        orch.print_json(failed);
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
