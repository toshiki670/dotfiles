//! AI-powered git commit with Conventional Commits。
//!
//! ステージ済み diff を `claude -p` に渡して Conventional Commits の JSON 提案を
//! 生成し、対話的に承認/修正して 1 つ以上のコミットを実行する。

mod claude;
mod execute;
mod git;
mod proposals;
mod ui;

use std::io::{self, Write};
use std::process::ExitCode;

use self::claude::call_claude;
use self::execute::execute;
use self::git::{git_capture, staged_files};
use self::ui::display;
use clap::Parser;

/// AI-powered git commit with Conventional Commits（claude -p）。
#[derive(Parser)]
#[command(
    name = "gcm",
    version,
    about = "AI-powered git commit with Conventional Commits (claude -p)"
)]
struct Cli {}

pub fn run() -> ExitCode {
    // 引数は取らないが、clap で --help / --version を提供する。
    let _ = Cli::parse();

    let staged = staged_files();
    if staged.is_empty() {
        eprintln!("ステージされた変更がありません。");
        eprintln!("git add でファイルをステージしてから実行してください。");
        return ExitCode::FAILURE;
    }

    let staged_str = staged.join(", ");
    let diff = git_capture(&["diff", "--cached"]).unwrap_or_default();

    let mut conversation = format!(
        "Propose commits for the following staged changes.\n\nStaged files: {staged_str}\n\ngit diff --staged:\n{diff}"
    );

    // 初回提案は haiku で生成。
    eprintln!("コミットを生成中...");
    let mut proposals = match call_claude(&conversation, "haiku") {
        Some(p) => p,
        None => return ExitCode::FAILURE,
    };

    // 対話承認ループ（Ctrl-C はプロセスを終了 = 中止）。
    loop {
        display(&proposals);

        print!("追加指示 (Enter でコミット実行 / Ctrl-C で中止): ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => {
                // EOF (Ctrl-D) などは中止扱い。
                println!();
                println!("中止しました。");
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
        }

        let instruction = line.trim();
        if instruction.is_empty() {
            return execute(&proposals);
        }

        let proposal_json = serde_json::to_string(&proposals).unwrap_or_default();
        conversation = format!(
            "{conversation}\n\nPrevious proposal (JSON): {proposal_json}\nRevision instruction: {instruction}\nRevise accordingly. Output ONLY the JSON array."
        );

        // 修正はより高品質な sonnet で。
        eprintln!("修正中...");
        proposals = match call_claude(&conversation, "sonnet") {
            Some(p) => p,
            None => return ExitCode::FAILURE,
        };
    }
}
