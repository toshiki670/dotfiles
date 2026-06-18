//! `fzf-gh`: Issue/PR を fzf で選び、種別に応じたアクションを選んで
//! `gh <group> <action> <number>` を stdout に 1 行出力する。fish shim
//! （`_fzf_gh.fish`）はそれをコマンドラインへ挿入する。
//!
//! fzf の UI・エラーメッセージは stderr に出し、stdout は組み立てたコマンド 1 行だけに
//! 保つ（`run_fzf` が stderr を継承する）。各段のキャンセル（ESC / Ctrl-C）や候補なしは
//! 何も出さずに正常終了する。

use std::process::ExitCode;

use fzf_picker::gh::{self, build_command, parse_selection};
use fzf_picker::launch::{command_exists, run_fzf};

// fish shim から引数なしで呼ばれるだけなので clap は持たない。

fn main() -> ExitCode {
    if !command_exists("gh") {
        eprintln!("fzf-gh: gh not found on PATH.");
        return ExitCode::FAILURE;
    }

    let candidates = match gh::list_candidates() {
        Ok(lines) => lines,
        Err(_) => {
            eprintln!("fzf-gh: failed to list issues/PRs.");
            return ExitCode::FAILURE;
        }
    };
    // open な Issue/PR が無い、または repo 外。静かに終わる。
    if candidates.is_empty() {
        return ExitCode::SUCCESS;
    }

    // 1 段目: Issue/PR の項目を選ぶ。
    let selection = match run_fzf(&candidates, &[]) {
        Ok(Some(selection)) => selection,
        Ok(None) => return ExitCode::SUCCESS, // キャンセル
        Err(_) => {
            eprintln!("fzf-gh: failed to run fzf.");
            return ExitCode::FAILURE;
        }
    };
    let Some((kind, number)) = parse_selection(&selection) else {
        return ExitCode::SUCCESS;
    };

    // 2 段目: 種別に応じたアクションを選ぶ。
    let actions: Vec<String> = kind.actions().iter().map(|a| (*a).to_string()).collect();
    let action = match run_fzf(&actions, &[]) {
        Ok(Some(action)) => action,
        Ok(None) => return ExitCode::SUCCESS, // キャンセル
        Err(_) => {
            eprintln!("fzf-gh: failed to run fzf.");
            return ExitCode::FAILURE;
        }
    };

    println!("{}", build_command(kind, &action, &number));
    ExitCode::SUCCESS
}
