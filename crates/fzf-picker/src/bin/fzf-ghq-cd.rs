//! 旧 `_fzf_ghq_cd`: `ghq list` の各 repo と、そのリンク worktree をツリー表示して
//! fzf で選択し、選択先の絶対パスを stdout に出力する。
//!
//! 親シェルの cd は別プロセスから行えないため、選択パスを stdout に出し、薄い fish
//! shim（`_fzf_ghq_cd.fish`）が `cd` する（shim 定石）。fzf 本体はこのバイナリが
//! TTY を継承して実行し、UI は fzf が `/dev/tty` に描く。stdout は選択パスのみ。

use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use clap::Parser;
use fzf_picker::{list_worktrees, picker_lines};

/// ghq list + リンク worktree を fzf で選び、選択先パスを stdout に出力する。
#[derive(Parser)]
#[command(
    name = "fzf-ghq-cd",
    version,
    about = "Pick a ghq repo or linked worktree with fzf and print its path"
)]
struct Cli {}

/// fzf の各行のフィールド区切り（表示は 1 列目のみ、cd 先は 3 列目）。
const TAB: char = '\t';

fn main() -> ExitCode {
    Cli::parse();

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

    match run_fzf(&lines) {
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
/// 行整形ロジックは純関数 `fzf_picker::picker_lines` に委ね、ここは IO（worktree
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

/// 候補行を stdin から流して fzf を起動し、選択行（あれば）を返す。
///
/// fzf は UI を `/dev/tty` に描くため stderr は継承で良い。stdout は捕捉して
/// 選択行を読む。キャンセル（exit code != 0）や空選択は `None`。
fn run_fzf(lines: &[String]) -> std::io::Result<Option<String>> {
    let mut child = Command::new("fzf")
        .args([
            "--delimiter",
            "\t",
            "--with-nth",
            "1",
            "--preview",
            "fish -c \"__fzf_picker_preview {2} {3} {4}\"",
            "--preview-window",
            "right:60%",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        for line in lines {
            writeln!(stdin, "{line}")?;
        }
        // stdin を drop して EOF を送る。
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let selection = String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string();
    Ok((!selection.is_empty()).then_some(selection))
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

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}
