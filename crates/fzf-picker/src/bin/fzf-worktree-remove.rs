//! 旧 `_fzf_worktree_remove`: リンク worktree を fzf で選び、確認のうえ削除する。
//!
//! 削除対象 worktree の **内側にいた場合だけ**、退避先（メイン worktree）の絶対パスを
//! stdout に出力する。fish shim はそのパスへ `cd` する（自分が消えるディレクトリから
//! 抜けるため）。それ以外の出力（プロンプト・結果メッセージ・git の出力）は stderr に
//! 出し、stdout の cd チャネルを汚さない。

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use clap::Parser;
use fzf_picker::format::removal_lines;
use fzf_picker::launch::run_fzf;
use fzf_picker::worktree::list_worktrees;

/// リンク worktree を fzf で選んで削除する。退避が要るときだけパスを stdout に出力。
#[derive(Parser)]
#[command(
    name = "fzf-worktree-remove",
    version,
    about = "Pick a linked git worktree with fzf and remove it"
)]
struct Cli {}

/// fzf 各行のフィールド区切り（表示は 1 列目、削除対象パスは 2 列目）。
const TAB: char = '\t';

fn main() -> ExitCode {
    Cli::parse();

    if !inside_work_tree() {
        eprintln!("not in a git repository");
        return ExitCode::FAILURE;
    }

    let worktrees = list_worktrees(Path::new("."));
    let main_path = worktrees.iter().find(|w| w.is_main).map(|w| w.path.clone());

    let lines = removal_lines(&worktrees);
    if lines.is_empty() {
        eprintln!("No worktrees to delete");
        return ExitCode::SUCCESS;
    }

    let selection = match run_fzf(
        &lines,
        &[
            "--preview",
            "git -C {2} log --oneline -20",
            "--preview-window",
            "right:60%",
        ],
    ) {
        Ok(Some(selection)) => selection,
        // ESC / Ctrl-C（fzf キャンセル）は何もせず正常終了。
        Ok(None) => return ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("fzf-worktree-remove: failed to run fzf.");
            return ExitCode::FAILURE;
        }
    };
    let Some(wpath) = selection.split(TAB).nth(1).map(str::to_string) else {
        return ExitCode::SUCCESS;
    };

    if !confirm("WT を削除しますか? [y/N] ") {
        return ExitCode::SUCCESS;
    }

    // 現在地が削除対象 worktree の内側なら、削除前に自プロセスを退避（git は現在の
    // worktree を消せない）し、親シェル用に退避先パスを stdout へ出す。削除の成否に
    // 関わらず（内側にいた以上）親シェルも退避させる（旧 fish と同じ）。
    let mut cd_target: Option<String> = None;
    if let Some(main) = &main_path
        && is_inside(&wpath)
    {
        let _ = std::env::set_current_dir(main);
        cd_target = Some(main.clone());
    }

    if worktree_remove(&wpath, false) {
        eprintln!("削除しました: {wpath}");
    } else if confirm("強制削除しますか? [y/N] ") && worktree_remove(&wpath, true) {
        eprintln!("強制削除しました: {wpath}");
    }

    if let Some(target) = cd_target {
        println!("{target}");
    }
    ExitCode::SUCCESS
}

/// `git rev-parse --is-inside-work-tree` 相当。
fn inside_work_tree() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// 現在地（cwd）が `wpath` と同じか、その配下にあるか。シンボリックリンクは
/// 解決して比較する（旧 fish の `path resolve` 相当）。
fn is_inside(wpath: &str) -> bool {
    let cur = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());
    let target = Path::new(wpath).canonicalize().ok();
    match (cur, target) {
        (Some(cur), Some(target)) => cur.starts_with(&target),
        _ => false,
    }
}

/// `git worktree remove [--force] <wpath>` を実行し成功可否を返す。git の出力は
/// stderr へ流して stdout（cd チャネル）を汚さない。
fn worktree_remove(wpath: &str, force: bool) -> bool {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "remove"]);
    if force {
        cmd.arg("--force");
    }
    match cmd.arg(wpath).stderr(Stdio::inherit()).output() {
        Ok(output) => {
            let _ = io::stderr().write_all(&output.stdout);
            output.status.success()
        }
        Err(_) => false,
    }
}

/// `[y/N]` プロンプトを stderr に出し、端末（stdin）から 1 行読んで `y`/`Y` 始まりかを
/// 返す（旧 fish の `read -P` + `string match -qri '^y'` 相当）。EOF は no 扱い。
fn confirm(prompt: &str) -> bool {
    eprint!("{prompt}");
    let _ = io::stderr().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
        return false;
    }
    line.trim().starts_with(['y', 'Y'])
}
