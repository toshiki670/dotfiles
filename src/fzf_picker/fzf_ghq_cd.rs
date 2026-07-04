//! `ghq list` の各 repo と、そのリンク worktree をツリー表示して fzf で選択し、
//! 選択先の絶対パスを stdout に出力する。
//!
//! 親シェルの cd は別プロセスから行えないため、選択パスを stdout に出し、薄い fish
//! shim（`_fzf_ghq_cd.fish`）が `cd` する（shim 定石）。fzf 本体はこのバイナリが
//! TTY を継承して実行し、UI は fzf が `/dev/tty` に描く。stdout は選択パスのみ。

use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use super::format::picker_lines;
use super::launch::{command_exists, run_fzf};
use super::worktree::list_worktrees;

/// fzf の各行のフィールド区切り（表示は 1 列目のみ、cd 先は 3 列目）。
const TAB: char = '\t';

// fish shim（`_fzf_ghq_cd.fish`）から引数なしで呼ばれるだけなので、引数パース（clap）
// は持たない。

pub fn run() -> ExitCode {
    if !command_exists("ghq") {
        eprintln!("fzf-ghq-cd: ghq command not found.");
        return ExitCode::from(127);
    }

    let Some(root) = capture(Command::new("ghq").arg("root")) else {
        eprintln!("fzf-ghq-cd: failed to resolve `ghq root`.");
        return ExitCode::FAILURE;
    };
    let Some(list) = capture(Command::new("ghq").arg("list")) else {
        eprintln!("fzf-ghq-cd: failed to run `ghq list`.");
        return ExitCode::FAILURE;
    };

    let lines = build_lines(&root, &list);

    let preview = "fish -c \"__fzf_picker_preview {2} {3} {4}\"";
    match run_fzf(
        &lines,
        &["--preview", preview, "--preview-window", "right:60%"],
    ) {
        Ok(Some(selection)) => {
            // 表示 / 種別 / パス / ghq 相対パス。cd 先は 3 列目（パス）。
            if let Some(path) = selection.split(TAB).nth(2) {
                println!("{path}");
            }
            ExitCode::SUCCESS
        }
        // ESC / Ctrl-C（fzf キャンセル）は「選択なし」として正常終了。
        Ok(None) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("fzf-ghq-cd: failed to run fzf.");
            ExitCode::FAILURE
        }
    }
}

/// `ghq list` の各 repo について worktree を引き、fzf 候補行を組み立てる。
/// 行整形ロジックは純関数 `super::picker_lines` に委ね、ここは IO（worktree
/// の取得）のみを担う。
fn build_lines(root: &str, list: &str) -> Vec<String> {
    let repos: Vec<(String, _)> = list
        .lines()
        .map(|rel| {
            let repo_path = format!("{root}/{rel}");
            (rel.to_string(), list_worktrees(Path::new(&repo_path)))
        })
        .collect();
    picker_lines(root, &repos)
}

/// 指定コマンドを実行し、成功時の stdout を trim して返す。失敗時は `None`。
/// コマンドの stderr は継承する。
fn capture(cmd: &mut Command) -> Option<String> {
    let output = cmd.stderr(Stdio::inherit()).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
